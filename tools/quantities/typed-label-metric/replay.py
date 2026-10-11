"""Offline replay only: authorized parquet columns and frozen sparse answers."""
import argparse
from collections import Counter
from fractions import Fraction
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
from metric import parse_gold, compare

HERE = Path(__file__).resolve().parent
GRADES = ('correct', 'refused', 'wrong', 'unscorable')
PARQUET = {
 'train': ('c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99',9013733),
 'validation': ('488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20',1350141)}
QHASH = 'bf0c0a3d095234fe1b00707e0d0b748477225acf4de1cb633ed6095b66e425f2'

def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path, obj): path.write_text(json.dumps(obj, indent=2, sort_keys=True)+'\n')
def counts(grades): return {g: grades.count(g) for g in GRADES}
def conservative(c): return dict(correct=c['correct'],refused=c['refused'],wrong=c['wrong']+c['unscorable'])

def main():
    ap=argparse.ArgumentParser()
    for name in ['data','frozen','output']: ap.add_argument('--'+name,type=Path,required=True)
    ap.add_argument('--sample',action='store_true')
    args=ap.parse_args()
    freeze=json.loads((HERE/'FREEZE.json').read_text())
    for name,h in freeze['files'].items(): assert sha(HERE/name)==h, ('freeze',name)
    for split,(h,n) in PARQUET.items():
        p=args.data/(split+'.parquet')
        assert sha(p)==h and p.stat().st_size==n, split
    pinned=json.loads((HERE/'INPUTS.json').read_text())
    legacy_path=HERE.parent/'evaluate_mathqa.py'
    assert sha(legacy_path)==pinned[legacy_path.name]['sha256']
    spec=importlib.util.spec_from_file_location('legacy_readonly',legacy_path)
    legacy=importlib.util.module_from_spec(spec); spec.loader.exec_module(legacy)
    import pyarrow.parquet as pq
    rows=pq.read_table(args.data/'validation.parquet',columns=['Problem','options','correct']).to_pylist()
    assert len(rows)==4475
    question_bytes=''.join(json.dumps(r['Problem'])+'\n' for r in rows).encode()
    qhash=hashlib.sha256(question_bytes).hexdigest()
    assert qhash==QHASH
    old=[legacy.numeric_option(r) for r in rows]
    assert sum(v is None for v in old)==1229
    reports={}; sources={}
    for name,n,expected in [('baseline',25,(22,4450,3)),('candidate',31,(23,4444,8))]:
        p=args.frozen/(name+'-full.json')
        assert sha(p)==pinned[p.name]['sha256'] and p.stat().st_size==pinned[p.name]['bytes']
        r=json.loads(p.read_text())
        assert r['total']==4475 and r['split']=='official-validation'
        assert (r['right'],r['refused'],r['wrong'])==expected
        assert r['questions_sha256']==qhash and r['non_numeric_gold']==1229
        records=r['answered']; indices=[a['index'] for a in records]
        assert len(records)==n and len(set(indices))==n and indices==sorted(indices)
        tally=Counter()
        for a in records:
            i=a['index']; assert type(i) is int and 0<=i<4475
            assert a['question']==rows[i]['Problem'] and a['actual']['status']=='answer'
            assert isinstance(a['actual']['value'],str)
            assert a['expected']==str(old[i])
            grade='right' if old[i] is not None and Fraction(a['actual']['value'])==old[i] else 'wrong'
            assert a['outcome']==grade
            tally[grade]+=1
        assert (tally['right'],4475-n,tally['wrong'])==expected
        reports[name]={a['index']:a for a in records}
        sources[name]=dict(**pinned[p.name],binary_sha256=r['binary_sha256'],legacy_counts=expected,answers=n)
    for i,a in reports['baseline'].items(): assert reports['candidate'][i]==a
    limit=100 if args.sample else 4475
    ledger=[]; changes=[]; gold_changes=[]; tallies={n:[] for n in reports}
    transitions={n:Counter() for n in reports}; reasons=Counter(); parse_transitions=Counter()
    for i,row in enumerate(rows[:limit]):
        selected,gold=parse_gold(row['options'],row['correct'])
        normalized=gold.record()
        reason=gold.reason; reasons[reason]+=1
        pt=('unsupported' if old[i] is None else 'scalar')+' -> '+gold.case
        parse_transitions[pt]+=1
        # Include type/dimension changes even when numerical coefficient is identical.
        changed=(old[i] is None)!=(gold.coefficient is None) or (old[i] is not None and
                 gold.coefficient is not None and old[i]!=gold.coefficient) or gold.case=='explicit_unit'
        item=dict(index=i,question_sha256=hashlib.sha256(row['Problem'].encode()).hexdigest(),
                  correct_key=row['correct'],selected=selected,legacy_value=str(old[i]),
                  gold=normalized,normalization_changed=changed,series={})
        if changed: gold_changes.append({k:v for k,v in item.items() if k!='series'})
        for name,answers in reports.items():
            record=answers.get(i)
            actual=dict(status='refusal') if record is None else record['actual']
            before='refused' if record is None else ('correct' if record['outcome']=='right' else 'wrong')
            result=compare(gold,actual)
            grade=result['grade']; tallies[name].append(grade)
            transitions[name][before+' -> '+grade]+=1
            item['series'][name]=dict(legacy_grade=before,actual=actual,**result)
            if before!=grade:
                changes.append(dict(index=i,series=name,before=before,after=grade,
                                    selected=selected,gold=normalized,actual=actual,comparison=result))
        ledger.append(item)
    assert [x['index'] for x in ledger]==list(range(limit))
    scores={n:counts(v) for n,v in tallies.items()}
    assert all(sum(v.values())==limit for v in scores.values())
    cases=Counter(x['gold']['case'] for x in ledger)
    case_scores={case:{n:counts([x['series'][n]['grade'] for x in ledger if x['gold']['case']==case])
                       for n in reports} for case in sorted(cases)}
    summary=dict(metric='exact-label-comparison-v3',official_mathqa_metric=False,
       comparability_break=True,question_requested_units_evaluated=False,
       scalar_case_ignores_prediction_dimension=True,total=limit,full_total=4475,
       full_questions_sha256=qhash,source=sources,parquet=PARQUET,
       legacy_unparseable_full=1229,scores=scores,
       conservative={n:conservative(v) for n,v in scores.items()},case_counts=dict(cases),
       case_scores=case_scores,gold_reasons=dict(reasons),parse_transitions=dict(parse_transitions),
       normalization_change_count=len(gold_changes),changed_grades=len(changes),
       legacy_to_v3={n:dict(v) for n,v in transitions.items()},
       baseline_to_candidate=dict(Counter(a+' -> '+b for a,b in zip(tallies['baseline'],tallies['candidate']))),
       freeze=freeze,new_source_data_bytes=0,
       limitations=['Sparse refusals inferred from indices and full-question hash; payloads unavailable.',
                    'Bare-label accuracy ignores prediction dimension.',
                    'Physical-label equality does not verify dataset/question semantics.',
                    'No candidate acceptance; parent review/rebaseline required.'])
    args.output.mkdir(parents=True,exist_ok=True)
    write(args.output/'summary.json',summary)
    write(args.output/'changed-grades.json',changes)
    (args.output/'ledger.jsonl').write_text(''.join(json.dumps(x,sort_keys=True)+'\n' for x in ledger))
    (args.output/'gold-normalization-changes.jsonl').write_text(''.join(json.dumps(x,sort_keys=True)+'\n' for x in gold_changes))
    originals=args.output/'originals'; originals.mkdir(exist_ok=True)
    for name in reports:
        p=args.frozen/(name+'-full.json'); dst=originals/p.name
        shutil.copyfile(p,dst)
        assert sha(p)==sha(dst)==pinned[p.name]['sha256']
    print(json.dumps({k:summary[k] for k in ['total','scores','conservative','case_counts','changed_grades','baseline_to_candidate']},indent=2))

if __name__=='__main__': main()
