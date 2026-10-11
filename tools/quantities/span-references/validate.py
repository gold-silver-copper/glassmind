"""Mechanical annotation checks only; semantic approval requires parent review."""
import argparse
from fractions import Fraction
import json
from pathlib import Path
import subprocess

ACCEPTED={'resolved','equivalent'}
STATUSES=ACCEPTED|{'ambiguous','unsupported','label_error'}
LIMIT=2*1024*1024


def require(ok,reason):
    if not ok:raise ValueError(reason)


def keys(obj,names):
    require(isinstance(obj,dict) and set(obj)==set(names.split()),'unexpected object fields')


def integer(x):return type(x) is int


def text(x,limit=4000):return isinstance(x,str) and bool(x.strip()) and len(x.encode())<=limit


def evidence(items,row):
    require(isinstance(items,list) and 1<=len(items)<=6,'evidence count')
    for e in items:
        keys(e,'field start end quote')
        require(e['field'] in ('question','answer'),'evidence field')
        raw=row[e['field']].encode()
        require(integer(e['start']) and integer(e['end']) and 0<=e['start']<e['end']<=len(raw),'evidence range')
        require(isinstance(e['quote'],str) and raw[e['start']:e['end']]==e['quote'].encode(),'evidence bytes')
    return items


def arg(source,row,leaf,constants):
    keys(source,'kind index');kind,index=source['kind'],source['index']
    require(kind in ('input','step','constant') and integer(index) and index>=0,'source reference')
    require(source in leaf['alternatives'],'source outside full original alternatives')
    if kind=='input':require(index<len(row['numbers']),'input range')
    elif kind=='constant':require(index<len(constants),'constant range')
    else:require(index<leaf['available_before'],'future source reference')
    return kind,index


def canonical(steps):
    out=[]
    for s in steps:
        pair=[(s[side]['kind'],s[side]['index']) for side in ('left','right')]
        if s['op'] in ('add','multiply'):pair.sort()
        out.append((s['op'],tuple(pair)))
    return tuple(out)


def validate(batch,annotations):
    keys(annotations,'schema rows');require(type(annotations['schema']) is int and annotations['schema']==1,'schema version')
    require(isinstance(annotations['rows'],list),'rows type')
    expected={row['id']:row for row in batch['rows']};rows=annotations['rows']
    require(len(expected)==len(batch['rows'])==len(rows),'row count')
    require(all(isinstance(x,dict) and isinstance(x.get('id'),str) for x in rows),'row IDs')
    require(len({x['id'] for x in rows})==len(rows) and {x['id'] for x in rows}==set(expected),'row identity')
    requests=[];records=[]
    for annotation in rows:
        keys(annotation,'id status reason evidence programs');row=expected[annotation['id']]
        require(annotation['status'] in STATUSES and text(annotation['reason']),'assessment')
        evidence(annotation['evidence'],row)
        alternatives=annotation['programs'];require(isinstance(alternatives,list),'programs type')
        if annotation['status'] not in ACCEPTED:
            require(not alternatives,'nonaccepted row has programs')
            records.append(dict(id=row['id'],status=annotation['status'],programs=[]));continue
        require(len(alternatives)==1 if annotation['status']=='resolved' else 2<=len(alternatives)<=4,'accepted program count')
        programs=[];canonical_forms=[];exact=set()
        for alternative in alternatives:
            keys(alternative,'assignments');assignments=alternative['assignments']
            require(isinstance(assignments,list) and len(assignments)==len(row['leaves']),'assignment count')
            mapping={}
            for assignment in assignments:
                keys(assignment,'leaf source role evidence');i=assignment['leaf']
                require(integer(i) and 0<=i<len(row['leaves']) and i not in mapping,'leaf identity')
                require(text(assignment['role'],1000),'semantic role text')
                leaf=row['leaves'][i];kind,index=arg(assignment['source'],row,leaf,batch['constants'])
                quoted=evidence(assignment['evidence'],row)
                if kind=='input':
                    n=row['numbers'][index]
                    require(any(e['field']=='question' and e['start']<=n['start'] and e['end']>=n['end'] for e in quoted),'input evidence misses occurrence')
                elif kind=='constant':
                    require(any(e['field']=='question' for e in quoted),'constant lacks question justification')
                else:
                    source_annotation=row['expression_nodes'][index]['annotation']
                    lo,hi=row['annotations'][source_annotation]['answer_char_span']
                    lo=len(row['answer'][:lo].encode());hi=len(row['answer'][:hi].encode())
                    require(any(e['field']=='answer' and e['start']<hi and e['end']>lo for e in quoted),'step evidence misses source calculation')
                mapping[i]=assignment['source']
            require(set(mapping)==set(range(len(row['leaves']))),'incomplete leaves')
            steps=[];values=[]
            for node in row['expression_nodes']:
                resolve=lambda side:mapping[side['leaf']] if 'leaf' in side else side
                step=dict(op=node['op'],left=resolve(node['left']),right=resolve(node['right']))
                def value(a):
                    k,i=a['kind'],a['index']
                    if k=='input':return Fraction(row['numbers'][i]['value'])
                    if k=='constant':return Fraction(batch['constants'][i])
                    require(i<len(values),'future direct subtree');return values[i]
                left,right=value(step['left']),value(step['right']);op=step['op']
                require(op in ('add','subtract','multiply','divide'),'operation')
                if op=='divide':require(right!=0,'division zero')
                answer={'add':lambda:left+right,'subtract':lambda:left-right,'multiply':lambda:left*right,'divide':lambda:left/right}[op]()
                require(answer==Fraction(node['value']),'changed annotated arithmetic')
                values.append(answer);steps.append(step)
            require(1<=len(steps)<=6 and values[-1]==Fraction(row['final_answer']),'final arithmetic')
            encoded=json.dumps(steps,sort_keys=True);require(encoded not in exact,'duplicate correlated program')
            exact.add(encoded);programs.append(dict(steps=steps));canonical_forms.append(canonical(steps))
        require(len(set(canonical_forms))==1,'program alternatives differ beyond direct commutation')
        for program in programs:requests.append(dict(question=row['question'],program=program))
        records.append(dict(id=row['id'],status=annotation['status'],programs=programs))
    return dict(rows=records,native_requests=requests,mechanically_accepted=sum(x['status'] in ACCEPTED for x in records),semantic_approval=False)


def main():
    p=argparse.ArgumentParser();p.add_argument('batch',type=Path);p.add_argument('annotations',type=Path);p.add_argument('--out',type=Path);p.add_argument('--executor',type=Path);a=p.parse_args()
    require(a.annotations.stat().st_size<=LIMIT,'annotation byte cap')
    batch=json.loads(a.batch.read_text());result=validate(batch,json.loads(a.annotations.read_text()))
    if a.executor:
        completed=subprocess.run([str(a.executor.resolve())],input=''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in result['native_requests']),text=True,capture_output=True,check=True,timeout=30)
        outputs=[json.loads(line) for line in completed.stdout.splitlines()];require(len(outputs)==len(result['native_requests']),'native output count')
        inputs={r['question']:r for r in batch['rows']}
        for request,output in zip(result['native_requests'],outputs,strict=True):
            row=inputs[request['question']]
            require(output['status']=='proposed' and Fraction(output['value'])==Fraction(row['final_answer']) and output['numbers']==row['numbers'],'native execution mismatch')
        result['native_outputs']=outputs
    if a.out:
        with a.out.open('x') as f:f.write(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(rows=len(result['rows']),mechanically_accepted=result['mechanically_accepted'],native_programs=len(result['native_requests']),semantic_approval=False)))


if __name__=='__main__':main()
