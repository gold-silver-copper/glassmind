import os
import collections,hashlib,json,sys
from pathlib import Path
ROOT=Path.cwd(); R=Path(os.environ.get('AUDIT_RUN',str(ROOT.parent/'groundingaudit-run/audit')));S=ROOT.parent/'groundingaudit-source';O=ROOT/'tools/quantities/grounding-audit'
sys.path.insert(0,str(R));from metric import parse_gold,compare
def read(p):return [json.loads(l) for l in p.read_text().splitlines()]
def dump(p,x):p.write_text(json.dumps(x,sort_keys=True,indent=2)+'\n')
def grade(row,out):
 actual=dict(status='answer',value=out['value'],unit=None) if out['status']=='proposed' else dict(status='refusal')
 return compare(parse_gold(row['options'],row['correct'])[1],actual)['grade']
clean=read(R/'clean.jsonl');outs=read(R/'diagnostics.jsonl');assert len(clean)==len(outs)==5772
assert read(R/'fit-free-original.jsonl')==[o['free'] for o in outs if o['split']=='FIT']
assert read(R/'replay200-helper.jsonl')==read(R/'replay200-original.jsonl')
calids=json.loads((S/'split.json').read_text())['calibration'];retained=dict(zip(calids,read(S/'calibration-candidate.jsonl')));assert len(retained)==5925
counts=collections.defaultdict(collections.Counter)
for r,o in zip(clean,outs):
 assert (r['index'],r['split'])==(o['index'],o['split'])
 assert o['diagnostic']['mode']=='explicit_train_annotation_oracles'
 if r['split']=='CAL':assert o['free']==retained[r['index']],r['index']
 if r['split']=='FIT':
  for e in o['diagnostic']['teacher']:
   key=('op',e['depth']) if e['head']==0 else ('arg',e['op'],e['head'])
   counts[key][e['target']]+=1
# Freeze FIT-only baseline counts before scoring CAL; no CAL target contributes.
dump(R/'majority-fit-counts.json',[dict(key=list(k),counts=dict(v)) for k,v in sorted(counts.items())])
metrics={}; strata={}; checks=collections.Counter(); perrows=[]
for tag in ['FIT','CAL']:
 agg=collections.Counter(); by=collections.defaultdict(lambda:[0,0]); byrows=collections.defaultdict(lambda:[0,0]);refusals=collections.defaultdict(collections.Counter)
 for r,o in zip(clean,outs):
  if r['split']!=tag:continue
  agg['rows']+=1; d=o['diagnostic'];gold=r['program'];depth=len(gold['steps']);n=d['input_count'];record=dict(index=r['index'],split=tag)
  assert d['forced_stop']==(depth<6) and d['forced_horizon']==(depth==6)
  agg['forced_stop']+=d['forced_stop'];agg['forced_horizon']+=d['forced_horizon'];checks['perturbed_gold_op_contexts']+=d['perturb_checks']
  for mode,out in [('free',o['free']),('oracle_plan',d['oracle'])]:
   result=grade(r,out);exact=out.get('program')==gold
   agg[mode+'_exact']+=exact;agg[mode+'_numeric_'+result]+=1
   if out['status']=='refusal':refusals[mode][out['reason']]+=1
   record[mode]=dict(exact=exact,grade=result)
   for axis,value in [('input_count',n),('length',depth)]:
    for metric,correct in [('exact',exact),('numeric',result=='correct')]:
     key=f'{mode}/{metric}/{axis}/{value}';by[key][0]+=int(correct);by[key][1]+=1
  for mode,events in [('teacher',d['teacher']),('oracle_args',d['oracle_events']),('majority',d['teacher'])]:
   rowall=collections.defaultdict(list); rowstrata=collections.defaultdict(list)
   for e in events:
    head=e['head'];actions=e.get('legal');target=e['target'];prediction=e['prediction']
    if mode=='majority':
     key=('op',e['depth']) if head==0 else ('arg',e['op'],head)
     prediction=max(actions,key=lambda a:counts[key][a]) # first mask item on ties
    if actions is not None:assert target in actions and prediction in actions
    if head>0:
     assert prediction < n or 16<=prediction<36 or 36<=prediction<36+e['depth']
    ok=prediction==target;rowall[head].append(ok)
    agg[f'{mode}_head{head}_decisions']+=1;agg[f'{mode}_head{head}_correct']+=ok
    for axis,value in [('step',e['depth']),('op',e['op']),('input_count',n)]:
     key=f'{mode}/head{head}/{axis}/{value}';by[key][0]+=ok;by[key][1]+=1;rowstrata[key].append(ok)
   for key,results in rowstrata.items():byrows[key][0]+=all(results);byrows[key][1]+=1
   for head,results in rowall.items():agg[f'{mode}_head{head}_rows_all']+=all(results)
   agg[f'{mode}_rows_all']+=all(x for a in rowall.values() for x in a)
   agg[f'{mode}_argument_rows_all']+=all(x for h,a in rowall.items() if h>0 for x in a)
   record[mode+'_all']={str(h):all(v) for h,v in rowall.items()}
  assert sum(e['target']==4 for e in d['teacher'] if e['head']==0)==(depth<6)
  assert len(d['teacher'])==3*depth+(depth<6)
  checks['rows_checked']+=1;perrows.append(record)
 metrics[tag]=dict(agg);metrics[tag]['refusal_reasons']={k:dict(v) for k,v in refusals.items()}
 strata[tag]={k:dict(correct=v[0],denominator=v[1],accuracy=v[0]/v[1],rows_all_correct=byrows[k][0] if k in byrows else None,rows=byrows[k][1] if k in byrows else None) for k,v in sorted(by.items())}
assert checks['rows_checked']==5772
checks.update(dict(exact_native_clean_FIT=4590,exact_helper_replay200=200,exact_retained_clean_CAL=1182,baseline_fit_rows=4590,baseline_cal_fit_rows=0,no_forward_references=5772,stop_denominators_checked=5772))
dump(R/'summary.json',dict(metrics=metrics,selfchecks=dict(checks),corpus=json.loads((R/'corpus-frozen.json').read_text())))
dump(R/'strata.json',strata)
(R/'per-row-metrics.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in perrows))
for name in ['summary.json','strata.json','corpus-frozen.json','native-verification.json']: (O/name).write_bytes((R/name).read_bytes())
print(json.dumps(metrics,indent=2))
