#!/usr/bin/env python3
"""Read authorized columns and frozen sparse reports; never execute inference."""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import importlib.util
import json
from pathlib import Path
from metric import parse_gold, select_option, scalar

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path,obj): path.write_text(json.dumps(obj,indent=2,sort_keys=True)+'\n')
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--data',type=Path,required=True); ap.add_argument('--frozen',type=Path,required=True); ap.add_argument('--output',type=Path,required=True); ap.add_argument('--sample',action='store_true'); args=ap.parse_args()
    freeze=json.loads((HERE/'FREEZE.json').read_text())
    for name,h in freeze['files'].items(): assert sha(HERE/name)==h,name
    import pyarrow.parquet as pq
    rows=pq.read_table(args.data/'validation.parquet',columns=['Problem','options','correct']).to_pylist()
    assert len(rows)==4475
    questions=''.join(json.dumps(r['Problem'])+'\n' for r in rows)
    qhash=hashlib.sha256(questions.encode()).hexdigest()
    legacy_path=HERE.parent/'evaluate_mathqa.py'
    spec=importlib.util.spec_from_file_location('legacy',legacy_path); legacy=importlib.util.module_from_spec(spec); spec.loader.exec_module(legacy)
    old=[legacy.numeric_option(r) for r in rows]; assert sum(x is None for x in old)==1229
    reports={}; source={}
    for name,n,count in [('baseline',25,(22,4450,3)),('candidate',31,(23,4444,8))]:
        path=args.frozen/(name+'-full.json'); r=json.loads(path.read_text())
        assert r['total']==4475 and r['split']=='official-validation'
        assert (r['right'],r['refused'],r['wrong'])==count and r['non_numeric_gold']==1229
        assert r['questions_sha256']==qhash
        records=r['answered']; assert len(records)==n
        indices=[x['index'] for x in records]; assert len(set(indices))==n and indices==sorted(indices)
        assert r['refused']==4475-n and r['right']+r['wrong']==n
        tally=Counter()
        for a in records:
            i=a['index']; assert type(i) is int and 0<=i<4475
            assert a['question']==rows[i]['Problem'] and a['actual']['status']=='answer'
            assert a['expected']==str(old[i])
            value=Fraction(a['actual']['value'])
            outcome='right' if old[i] is not None and value==old[i] else 'wrong'
            assert a['outcome']==outcome; tally[outcome]+=1
        assert (tally['right'],tally['wrong'])==(r['right'],r['wrong'])
        reports[name]={a['index']:a for a in records}
        source[name]=dict(path=str(path),bytes=path.stat().st_size,sha256=sha(path),binary_sha256=r['binary_sha256'],historical_counts=count,answered=n)
    # Frozen baseline records must be unchanged in rejected candidate.
    for i,a in reports['baseline'].items(): assert reports['candidate'][i]==a
    indexed=list(enumerate(rows))[:100] if args.sample else list(enumerate(rows))
    audit=[]; outcomes={name:[] for name in reports}; changed=[]
    old_none_categories=Counter(); parse_transitions=Counter(); reasons=Counter()
    for i,row in indexed:
        try:
            selected=select_option(row['options'],row['correct']); parsed=parse_gold(selected,row['Problem'])
        except ValueError as e:
            selected=None; parsed=dict(value=None,reason='option_binding:'+str(e),gold_unit=None,requested_unit=None,raw_value=None)
        value=parsed['value']; reasons[parsed['reason']]+=1
        transition=('unparseable' if old[i] is None else 'parseable')+' -> '+('unparseable' if value is None else 'parseable')
        parse_transitions[transition]+=1
        if old[i] is None: old_none_categories[parsed['reason']]+=1
        item=dict(index=i,question_sha256=hashlib.sha256(row['Problem'].encode()).hexdigest(),correct_key=row['correct'],selected=selected,legacy_value=str(old[i]),**parsed)
        for name,answers in reports.items():
            a=answers.get(i)
            actual_value=None
            if a is None: grade='refused'; before='refused'
            else:
                before='correct' if a['outcome']=='right' else 'wrong'
                if value is None: grade='unscorable'
                else:
                    actual=a['actual']; unit=actual.get('unit')
                    if unit:
                        suffix='%' if unit=='percent' else unit
                        normalized=parse_gold(actual['value']+' '+suffix,row['Problem'])
                        actual_value=normalized['value']
                    else:
                        v=scalar(actual['value']); actual_value=str(v) if v is not None else None
                    grade='correct' if actual_value is not None and Fraction(actual_value)==Fraction(value) else 'wrong'
                if before!=grade:
                    changed.append(dict(series=name,index=i,question=row['Problem'],before=before,after=grade,selected=selected,normalization=parsed,actual=a['actual'],normalized_actual=actual_value))
            outcomes[name].append(grade)
            item[name]=grade
        audit.append(item)
    summary=dict(metric='mathqa-exact-explicit-units-v2',total=len(indexed),full_questions_sha256=qhash,legacy_unparseable_full=1229,parse_transitions=dict(parse_transitions),all_gold_categories=dict(reasons),legacy_unparseable_categories=dict(old_none_categories),unscorable_gold=sum(x['value'] is None for x in audit),source=source,legacy_adapter_sha256=sha(legacy_path),freeze=freeze)
    summary['scores']={name:{g:grades.count(g) for g in ['correct','refused','wrong','unscorable']} for name,grades in outcomes.items()}
    assert all(sum(v.values())==len(indexed) for v in summary['scores'].values())
    summary['baseline_to_candidate_transitions']=dict(Counter(a+' -> '+b for a,b in zip(outcomes['baseline'],outcomes['candidate'])))
    summary['legacy_to_v2_transitions']={name:dict(Counter(('refused' if i not in reports[name] else ('correct' if reports[name][i]['outcome']=='right' else 'wrong'))+' -> '+grade for (i,_),grade in zip(indexed,outcomes[name]))) for name in reports}
    summary['new_data_bytes']=0
    summary['parquet']={split:dict(bytes=(args.data/(split+'.parquet')).stat().st_size,sha256=sha(args.data/(split+'.parquet'))) for split in ['train','validation']}
    args.output.mkdir(parents=True,exist_ok=True)
    write(args.output/'summary.json',summary); write(args.output/'changed-grades.json',changed)
    (args.output/'gold-audit.jsonl').write_text(''.join(json.dumps(x,sort_keys=True)+'\n' for x in audit))
    print(json.dumps(summary,indent=2))
if __name__=='__main__': main()
