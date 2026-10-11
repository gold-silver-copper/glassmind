#!/usr/bin/env python3
"""New multi-frame adapter, calling the unchanged pinned upstream metric.

Inference receives ONLY serialized speaker/utterance arrays. Every service state
is emitted on every user turn. Gold frames are consulted only after inference.
"""
import argparse, hashlib, json, pathlib, subprocess, sys, warnings
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[2]/'crates/dialogue/tools'))
with warnings.catch_warnings():
 warnings.simplefilter('ignore')
 import upstream_metrics as metrics
p=argparse.ArgumentParser()
p.add_argument('--binary',required=True);p.add_argument('--model',required=True)
p.add_argument('--data',required=True);p.add_argument('--schema',required=True)
p.add_argument('--output',required=True,type=pathlib.Path)
p.add_argument('--sample',choices=['first','multi']);p.add_argument('--audit',action='store_true')
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
ds=sorted(json.load(open(a.data)),key=lambda d:d['dialogue_id'])
if a.sample=='first': ds=ds[:10]
if a.sample=='multi': ds=[d for d in ds if len(d['services'])>1][:10]
schemas={s['service_name']:s for s in json.load(open(a.schema))}
assert all(set(d['services'])<=schemas.keys() for d in ds)
inputs=[[{k:t[k] for k in ('speaker','utterance')} for t in d['turns']] for d in ds]
input_path=a.output/'inputs.json';input_path.write_text(json.dumps(inputs))
pred_path=a.output/'predictions.json'
cmd=[a.binary,'predict','--schema',a.schema,'--model',a.model,'--input',str(input_path),'--output',str(pred_path)]
subprocess.run(cmd,check=True)
pred=json.loads(pred_path.read_text());assert len(pred)==len(ds)

def score(subset):
 r={'dialogues':0,'user_turns':0,'gold_frames':0,'predicted_frames':0,'tp':0,'fp':0,'fn':0,'route_exact':0,'strict_frame_correct':0,'intent_correct':0,'full_joint_correct':0,'fuzzy_joint_sum':0.0,'gold_switches':0,'correct_switches':0,'service':{}}
 for d,ps in zip(ds,pred):
  if subset=='multi' and len(d['services'])==1:continue
  if subset=='single' and len(d['services'])!=1:continue
  r['dialogues']+=1;last_gold=None;last_pred=None
  by_index={x['turn_index']:x for x in ps};assert len(by_index)==len(ps)
  for i,t in enumerate(d['turns']):
   if t['speaker']!='USER': continue
   s=by_index.pop(i);fs={f['service']:f for f in s['frames']};assert set(fs)==schemas.keys() and len(fs)==len(s['frames'])
   gold={f['service']:f for f in t['frames']};assert set(gold)<=schemas.keys()
   active=set(s['active_services']);assert active<=schemas.keys()
   correct_route=active==set(gold)
   r['user_turns']+=1;r['gold_frames']+=len(gold);r['predicted_frames']+=len(fs)
   r['tp']+=len(active&gold.keys());r['fp']+=len(active-gold.keys());r['fn']+=len(gold.keys()-active);r['route_exact']+=correct_route
   if last_gold is not None and set(gold)!=last_gold:
    r['gold_switches']+=1;r['correct_switches']+=correct_route and last_pred==last_gold
   last_gold=set(gold);last_pred=active
   exact=True
   for service,g in gold.items():
    f=fs[service];schema=schemas[service]
    strict=metrics.get_average_and_joint_goal_accuracy(g,f,schema,False)['joint_goal_accuracy']
    fuzzy=metrics.get_average_and_joint_goal_accuracy(g,f,schema,True)['joint_goal_accuracy']
    intent=metrics.get_active_intent_accuracy(g,f)
    r['strict_frame_correct']+=int(strict);r['fuzzy_joint_sum']+=float(fuzzy);r['intent_correct']+=int(intent)
    exact=exact and strict==1 and intent==1
    sr=r['service'].setdefault(service,{'frames':0,'strict':0,'fuzzy_sum':0.0})
    sr['frames']+=1;sr['strict']+=int(strict);sr['fuzzy_sum']+=float(fuzzy)
   r['full_joint_correct']+=bool(exact and correct_route)
  assert not by_index
 r['active_precision']=r['tp']/(r['tp']+r['fp']) if r['tp']+r['fp'] else 0
 r['active_recall']=r['tp']/(r['tp']+r['fn']) if r['tp']+r['fn'] else 0
 r['active_f1']=2*r['tp']/(2*r['tp']+r['fp']+r['fn']) if r['gold_frames'] else 0
 return r
report={'model_sha256':hashlib.sha256(pathlib.Path(a.model).read_bytes()).hexdigest(),'ids':[d['dialogue_id'] for d in ds],'all':score('all'),'single':score('single'),'multi':score('multi')}
if a.audit:
 reverse_path=a.output/'reverse-inputs.json';reverse_path.write_text(json.dumps(inputs[::-1]))
 reverse_pred=a.output/'reverse-predictions.json'
 reverse_cmd=cmd.copy();reverse_cmd[reverse_cmd.index('--input')+1]=str(reverse_path);reverse_cmd[reverse_cmd.index('--output')+1]=str(reverse_pred)
 subprocess.run(reverse_cmd,check=True)
 assert json.loads(reverse_pred.read_text())==pred[::-1], 'conversation reset/order leak'
 poison_path=a.output/'annotation-input.json';poison_path.write_text(json.dumps([[{'speaker':'USER','utterance':'hello','frames':[]}]]))
 reverse_cmd[reverse_cmd.index('--input')+1]=str(poison_path)
 rejected=subprocess.run(reverse_cmd,capture_output=True)
 assert rejected.returncode!=0 and b'unknown field' in rejected.stderr
 report['isolation']={'utterance_only':True,'reversed_batch_identical':True,'annotation_input_rejected':True,'conversations':len(ds),'gold_routing':False}
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='ids'},indent=2))
