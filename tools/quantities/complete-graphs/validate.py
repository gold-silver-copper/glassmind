"""Compile complete quantity graphs; semantic approval always needs parent review."""
import argparse
from fractions import Fraction
import json
from pathlib import Path
import re
import subprocess

CONSTANTS=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
STATUSES={'resolved','ambiguous','unsupported','label_error'}
OPS={'add','subtract','multiply','divide'}
LIMIT=2*1024**2


def require(condition,message):
    if not condition:raise ValueError(message)


def fields(obj,names):
    require(isinstance(obj,dict) and set(obj)==set(names.split()),'object fields')


def integer(value):return type(value) is int


def prose(value,limit=4000):
    return isinstance(value,str) and bool(value.strip()) and len(value.encode())<=limit


def rational(value):
    require(isinstance(value,str) and len(value)<=128 and re.fullmatch(r'[+-]?(?:[0-9]+(?:\.[0-9]+)?|\.[0-9]+)(?:/[0-9]+)?',value) is not None,'rational value')
    try:return Fraction(value)
    except (ValueError,ZeroDivisionError) as error:raise ValueError('rational value') from error


def citations(items,row,question_only=False):
    require(isinstance(items,list) and 1<=len(items)<=6,'evidence count')
    for item in items:
        fields(item,'field start end quote')
        require(isinstance(item['field'],str) and item['field'] in ('question','answer'),'evidence field')
        require(not question_only or item['field']=='question','question evidence required')
        raw=row[item['field']].encode();lo,hi=item['start'],item['end']
        require(integer(lo) and integer(hi) and 0<=lo<hi<=len(raw),'evidence range')
        require(isinstance(item['quote'],str) and item['quote'].encode()==raw[lo:hi],'evidence bytes')
    return items


def covers(evidence,number):
    return any(e['field']=='question' and e['start']<=number['start'] and e['end']>=number['end'] for e in evidence)


def unique_keys(pairs):
    result={}
    for key,value in pairs:
        require(key not in result,'duplicate JSON key')
        result[key]=value
    return result


def load(path):
    require(path.stat().st_size<=LIMIT,'annotation byte cap')
    return json.loads(path.read_text(),object_pairs_hook=unique_keys)


def validate(batch,annotations):
    require(batch['constants']==CONSTANTS,'native constants changed')
    fields(annotations,'schema rows')
    require(integer(annotations['schema']) and annotations['schema']==2,'schema version')
    source={r['id']:r for r in batch['rows']};rows=annotations['rows']
    require(isinstance(rows,list) and len(source)==len(batch['rows'])==len(rows),'row count')
    require(all(isinstance(r,dict) and isinstance(r.get('id'),str) for r in rows),'row IDs')
    require(len({r['id'] for r in rows})==len(rows) and {r['id'] for r in rows}==set(source),'row identity')
    records=[];requests=[];checks=[]
    for annotation in rows:
        fields(annotation,'id status reason evidence rationale_issues quantities steps')
        row=source[annotation['id']];numbers=row['numbers']
        require(len(numbers)<=16,'native input bound')
        require(isinstance(annotation['status'],str) and annotation['status'] in STATUSES,'status')
        require(prose(annotation['reason']),'reason')
        citations(annotation['evidence'],row)
        issues=annotation['rationale_issues']
        require(isinstance(issues,list) and len(issues)<=8 and all(prose(x,2000) for x in issues),'rationale issues')
        quantities,steps=annotation['quantities'],annotation['steps']
        require(isinstance(quantities,list) and isinstance(steps,list),'graph lists')
        if annotation['status']!='resolved':
            require(not quantities and not steps,'nonaccepted graph')
            records.append(dict(id=row['id'],status=annotation['status'],program=None));continue
        require(len(quantities)<=16 and 1<=len(steps)<=6,'native graph bounds')
        groups=[];seen_mentions=set()
        for quantity in quantities:
            fields(quantity,'mentions meaning unit coreference evidence')
            mentions=quantity['mentions']
            require(isinstance(mentions,list) and 1<=len(mentions)<=16,'mention list')
            require(all(integer(i) and 0<=i<len(numbers) for i in mentions),'mention index')
            require(len(set(mentions))==len(mentions),'duplicate mention')
            require(not seen_mentions.intersection(mentions),'overlapping groups')
            seen_mentions.update(mentions)
            require(prose(quantity['meaning'],1000) and prose(quantity['unit'],200),'quantity semantics')
            explanation=quantity['coreference']
            require(isinstance(explanation,str) and len(explanation.encode())<=2000 and (len(mentions)==1 or bool(explanation.strip())),'coreference explanation')
            evidence=citations(quantity['evidence'],row,True)
            require(all(covers(evidence,numbers[i]) for i in mentions),'uncited mention')
            values={rational(numbers[i]['value']) for i in mentions}
            require(len(values)==1,'unequal coreference values')
            groups.append(dict(mentions=sorted(mentions),value=next(iter(values))))
        native=[];values=[];used_quantities=set();dependencies=[]
        for step_index,step in enumerate(steps):
            fields(step,'op left right meaning unit value evidence')
            require(isinstance(step['op'],str) and step['op'] in OPS,'operation')
            require(prose(step['meaning'],1000) and prose(step['unit'],200),'step semantics')
            citations(step['evidence'],row,True)
            dependencies.append(set());native_step=dict(op=step['op']);operands=[]
            for side in ('left','right'):
                operand=step[side];fields(operand,'source role evidence');reference=operand['source']
                fields(reference,'kind index');kind,index=reference['kind'],reference['index']
                require(isinstance(kind,str) and kind in ('quantity','step','constant') and integer(index) and index>=0,'source reference')
                require(prose(operand['role'],1000),'operand role')
                evidence=citations(operand['evidence'],row,True)
                if kind=='quantity':
                    require(index<len(groups),'quantity reference')
                    group=groups[index]
                    require(any(covers(evidence,numbers[i]) for i in group['mentions']),'operand misses quantity')
                    used_quantities.add(index);value=group['value'];compiled=dict(kind='input',index=group['mentions'][0])
                elif kind=='step':
                    require(index<step_index,'future step reference')
                    dependencies[-1].add(index);value=values[index];compiled=dict(kind='step',index=index)
                else:
                    require(index<len(CONSTANTS),'constant reference')
                    value=rational(CONSTANTS[index]);compiled=dict(kind='constant',index=index)
                native_step[side]=compiled;operands.append(value)
            left,right=operands;op=step['op']
            require(op!='divide' or right!=0,'division zero')
            result={'add':lambda:left+right,'subtract':lambda:left-right,'multiply':lambda:left*right,'divide':lambda:left/right}[op]()
            require(result==rational(step['value']),'intermediate arithmetic')
            values.append(result);native.append(native_step)
        require(used_quantities==set(range(len(groups))),'unused quantity')
        reached=set();pending=[len(steps)-1]
        while pending:
            i=pending.pop()
            if i in reached:continue
            reached.add(i);pending.extend(dependencies[i])
        require(reached==set(range(len(steps))),'dead step')
        require(values[-1]==rational(row['final_answer']),'reference final mismatch')
        # Native execution checks every prefix, not only a numerically matching final.
        for i,value in enumerate(values):
            requests.append(dict(question=row['question'],program=dict(steps=native[:i+1])))
            checks.append(dict(id=row['id'],step=i,expected=str(value),numbers=numbers))
        records.append(dict(id=row['id'],status='resolved',program=dict(steps=native),quantity_mentions=[g['mentions'] for g in groups],values=[str(v) for v in values]))
    return dict(rows=records,native_requests=requests,native_checks=checks,mechanically_accepted=sum(r['status']=='resolved' for r in records),semantic_approval=False)


def execute(result,executor):
    completed=subprocess.run([str(executor.resolve())],input=''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in result['native_requests']),text=True,capture_output=True,check=True,timeout=30)
    outputs=[json.loads(line) for line in completed.stdout.splitlines()]
    require(len(outputs)==len(result['native_checks']),'native output count')
    for request,check,output in zip(result['native_requests'],result['native_checks'],outputs,strict=True):
        require(output['status']=='proposed' and rational(output['value'])==rational(check['expected']) and output['numbers']==check['numbers'] and output['program']==request['program'],'native execution mismatch')
    result['native_outputs']=outputs


def main():
    parser=argparse.ArgumentParser();parser.add_argument('batch',type=Path);parser.add_argument('annotations',type=Path);parser.add_argument('--out',type=Path);parser.add_argument('--executor',type=Path);args=parser.parse_args()
    result=validate(json.loads(args.batch.read_text()),load(args.annotations))
    if args.executor:execute(result,args.executor)
    if args.out:
        with args.out.open('x') as output:json.dump(result,output,ensure_ascii=False,indent=2);output.write('\n')
    print(json.dumps(dict(rows=len(result['rows']),mechanically_accepted=result['mechanically_accepted'],native_prefixes=len(result['native_requests']),semantic_approval=False)))


if __name__=='__main__':main()
