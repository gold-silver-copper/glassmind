#!/usr/bin/env python3
"""Report every development correctness transition; no filtering of failures."""
import argparse,collections,json,pathlib,sys,warnings
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[2]/'crates/dialogue/tools'))
with warnings.catch_warnings():
 warnings.simplefilter('ignore')
 import upstream_metrics as metrics
p=argparse.ArgumentParser()
for arg in ('data','schema','baseline','candidate','output'):p.add_argument('--'+arg,required=True)
a=p.parse_args()
ds=sorted(json.load(open(a.data)),key=lambda d:d['dialogue_id']);schemas={s['service_name']:s for s in json.load(open(a.schema))}
bs=json.load(open(a.baseline));cs=json.load(open(a.candidate));assert len(ds)==len(bs)==len(cs)
r={'joint_transitions':collections.Counter(),'frame_transitions':collections.Counter(),'routing_transitions':collections.Counter(),'changed_joint_ids':[]}
def outcome(p,t):
 fs={f['service']:f for f in p['frames']};gold={f['service']:f for f in t['frames']}
 frame_ok={s:metrics.get_average_and_joint_goal_accuracy(g,fs[s],schemas[s],False)['joint_goal_accuracy']==1 for s,g in gold.items()}
 route=set(p['active_services'])==set(gold)
 full=route and all(frame_ok.values()) and all(metrics.get_active_intent_accuracy(g,fs[s])==1 for s,g in gold.items())
 return ('right' if full else 'refused' if not p['active_services'] else 'wrong'),frame_ok,route
for d,b,c in zip(ds,bs,cs):
 assert len(b)==len(c)
 for x,y in zip(b,c):
  assert x['turn_index']==y['turn_index'];i=x['turn_index'];t=d['turns'][i]
  bo,bf,br=outcome(x,t);co,cf,cr=outcome(y,t)
  r['joint_transitions'][bo+'->'+co]+=1
  r['routing_transitions'][str(br)+'->'+str(cr)]+=1
  for s in bf:r['frame_transitions'][str(bf[s])+'->'+str(cf[s])]+=1
  if bo!=co:r['changed_joint_ids'].append([d['dialogue_id'],i,bo,co])
pathlib.Path(a.output).write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k!='changed_joint_ids'},indent=2))
