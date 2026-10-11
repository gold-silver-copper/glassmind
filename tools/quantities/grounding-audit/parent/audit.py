"""Independent reconstruction from TRAIN rows and retained native diagnostic events."""
from pathlib import Path
import collections,hashlib,json,re,sys
import pyarrow.parquet as pq
w=Path('/home/gsc/.cache/glassmind/continuous-20261010');src=w/'groundingaudit-source';f=w/'groundingaudit-run/frozen-candidate';d=f/'audit';out=Path(__file__).parent
read=lambda p:[json.loads(s) for s in p.read_text().splitlines()]
rows=pq.read_table(src/'train.parquet',columns=['Problem']).to_pylist();groups=[hashlib.sha256(re.sub('[0-9]+','#',' '.join(r['Problem'].lower().split())).encode()).hexdigest() for r in rows]
split=json.loads((src/'split.json').read_text());assert groups==split['group_hashes'];assert [i for i,g in enumerate(groups) if int(g,16)%5]==split['fit'];assert [i for i,g in enumerate(groups) if int(g,16)%5==0]==split['calibration']
members=read(d/'membership.jsonl');assert len(members)==len(rows)==29837
for i,r in enumerate(members):assert r['index']==i and r['group']==groups[i] and r['split']==('CAL' if int(groups[i],16)%5==0 else 'FIT')
clean=read(d/'clean.jsonl');assert {r['index'] for r in clean}=={r['index'] for r in members if r['clean']}
assert read(out/'replay200.jsonl')==read(d/'replay200-expected.jsonl');assert len(read(out/'replay200.jsonl'))==200
assert read(d/'gold-all-original.jsonl')==read(src/'train-executions.jsonl')
assert read(d/'fit-free-original.jsonl')==[r['free'] for r in read(d/'diagnostics.jsonl') if r['split']=='FIT']
sys.path.insert(0,str(d));from metric import compare,parse_gold
diagnostics=read(d/'diagnostics.jsonl');totals=collections.defaultdict(collections.Counter)
for c,r in zip(clean,diagnostics,strict=True):
 assert c['index']==r['index'] and c['split']==r['split'];assert c['question']==rows[c['index']]['Problem'];t=totals[c['split']];t['rows']+=1;v=r['diagnostic'];assert v['mode']=='explicit_train_annotation_oracles'
 t['forced_stop']+=v['forced_stop'];t['forced_horizon']+=v['forced_horizon'];assert v['forced_stop']==(len(c['program']['steps'])<6)
 for name,pred in [('free',r['free']),('oracle_plan',v['oracle'])]:
  t[name+'_exact']+=pred.get('program')==c['program'];actual={'status':'answer','value':pred['value'],'unit':None} if pred['status']=='proposed' else {'status':'refusal'};grade=compare(parse_gold(c['options'],c['correct'])[1],actual)['grade'];t[name+'_numeric_'+grade]+=1
 for e in v['teacher']:
  h=e['head'];t[f'teacher_head{h}_decisions']+=1;t[f'teacher_head{h}_correct']+=e['target']==e['prediction'];assert e['prediction'] in e['legal'] and e['target'] in e['legal']
  if h==0 and e['target']!=4:t['nonstop_total']+=1;t['nonstop_correct']+=e['target']==e['prediction']
summary=json.loads((d/'summary.json').read_text())
for split,count in totals.items():
 for key,value in count.items():
  if not key.startswith('nonstop'):assert summary['metrics'][split][key]==value,(split,key,value)
report=dict(train_rows=len(rows),groups_reconstructed=True,clean_rows=len(clean),native200_exact=True,gold29837_exact=True,free4590_exact=True,totals=totals,scope='TRAIN diagnostic with explicit oracle conditions; model remains rejected')
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
