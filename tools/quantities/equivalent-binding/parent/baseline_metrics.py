"""Regrade archived original TRAIN diagnostics against the frozen formal oracle."""
from pathlib import Path
from collections import Counter
import hashlib,json,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parent.parent
metric=Path('/home/gsc/code/glassmind/tools/quantities/typed-label-metric/metric.py')
assert hashlib.sha256(metric.read_bytes()).hexdigest()=='24f1d65ed2f1785b940d7c674565dd9b49c168e498f57c23cbd6c8c2c6eab6f0'
sys.path.insert(0,str(metric.parent));from metric import compare,parse_gold
read=lambda p:[json.loads(s) for s in p.read_text().splitlines()]
archive=ROOT/'groundingaudit-run/frozen-candidate';m=json.loads((archive/'manifest.json').read_text())['files']
for name in ['audit/clean.jsonl','audit/diagnostics.jsonl']:
    assert hashlib.sha256((archive/name).read_bytes()).hexdigest()==m[name]['sha256']
clean={r['index']:r for r in read(archive/'audit/clean.jsonl')}
targets={r['index']:r for r in read(HERE/'corpus/targets.jsonl')}
totals={s:{k:Counter() for k in ['free','oracle','teacher']} for s in ['FIT','CAL']}
graded=[]
for row in read(archive/'audit/diagnostics.jsonl'):
    c=clean[row['index']];t=targets[row['index']];assert row['split']==c['split']==t['split']
    total=totals[c['split']];label=parse_gold(c['options'],c['correct'])[1];item=dict(index=c['index'],split=c['split'])
    for mode,pred in [('free',row['free']),('oracle',row['diagnostic']['oracle'])]:
        answer=dict(status='answer',value=pred['value'],unit=None) if pred['status']=='proposed' else dict(status='refusal')
        grade=compare(label,answer)['grade'];total[mode][grade]+=1;total[mode]['exact_ast']+=pred.get('program')==c['program'];item[mode]=grade
    for e in row['diagnostic']['teacher']:
        head=e['head'];depth=e['depth']
        if head==0:continue
        step=t['steps'][depth];pairs=step['pairs'];key='left' if head==1 else 'right'
        assert e['target']==step['gold_'+key]
        accepted={p[0] for p in pairs} if head==1 else {r for l,r in pairs if l==step['gold_left']}
        total['teacher'][key+'_decisions']+=1
        total['teacher'][key+'_exact']+=e['prediction']==e['target']
        total['teacher'][key+'_equivalent']+=e['prediction'] in accepted
    graded.append(item)
cal=totals['CAL'];assert cal['free']['correct']==143 and cal['oracle']['correct']==221
assert cal['teacher']['left_exact']==2197 and cal['teacher']['right_exact']==2499
assert cal['teacher']['left_decisions']==cal['teacher']['right_decisions']==4138
out=HERE/'corpus';(out/'baseline-grades.jsonl').write_text(''.join(json.dumps(g)+'\n' for g in graded))
report=dict(scope='New formal metric on frozen original TRAIN diagnostics; historical numeric scores unchanged',totals={s:{k:dict(v) for k,v in t.items()} for s,t in totals.items()})
(out/'baseline-metrics.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
