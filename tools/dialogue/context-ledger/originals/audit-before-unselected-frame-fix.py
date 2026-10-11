#!/usr/bin/env python3
"""Independent CAL decoder/overlay audit using unchanged upstream metrics."""
import collections, copy, hashlib, itertools, json, pathlib, sys, warnings
HERE=pathlib.Path(__file__).resolve().parent; A=HERE/'artifacts'
sys.path.insert(0,str(HERE.parents[2]/'crates/dialogue/tools'))
with warnings.catch_warnings():
 warnings.simplefilter('ignore')
 import upstream_metrics as metrics
S=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/entityledger-source')
def load(p): return json.loads(p.read_text())
def save(n,v): (A/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
data=load(A/'dataset.json'); ds=[d for d in data['dialogues'] if d['cal']]; schemas={s['service_name']:s for s in load(S/'schema.json')}
assert len(ds)==642
rows={}; total=collections.Counter(); source=collections.defaultdict(collections.Counter)
model=load(A/'model.json'); assert len((A/'model.json').read_bytes())<=16*1024*1024
# Prove only the boundary was inherited and its serialized mathematical values unchanged.
old=load(pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/contextledger-source/rejected-model.json'))
assert model['boundary']==old['boundary'];del old
for line in open(A/'source-spans.jsonl'):
 z=json.loads(line);r=z['row']; split='CAL' if r['cal'] else 'FIT';c=source[split];c['spans']+=1
 exact=set(r['exact_roles']);label=next(iter(exact)) if len(exact)==1 and next(iter(exact)) in model['roles'] else 'NONE'
 assert label==r['label']; scores=z['scores']; prediction=max(range(len(scores)),key=lambda i:scores[i]);assert prediction==z['prediction']
 assert abs(z['margin']-(scores[prediction]-max(s for i,s in enumerate(scores) if i!=prediction)))<0.002
 pred=model['roles'][prediction];c['gold_none']+=label=='NONE';c['pred_none']+=pred=='NONE';c['correct']+=label==pred
 c['positive_correct']+=pred==label and pred!='NONE';c['positive_wrong']+=pred!=label and pred!='NONE'
 c['absent_or_unaligned']+=not exact;c['ambiguous']+=len(exact)>1;c['out_of_label']+=len(exact)==1 and label=='NONE'
 assert z['enters_ledger']==(pred!='NONE')
for line in open(A/'proposals.jsonl'):
 r=json.loads(line); total['CAL_opportunities' if r['cal'] else 'FIT_opportunities']+=1
 total['CAL_candidates' if r['cal'] else 'FIT_candidates']+=len(r['candidates'])
 total['empty_opportunities']+=not r['candidates'];total['negative_opportunities']+=not any(c['correct'] for c in r['candidates'])
 if r['cal']: rows[(r['id'],r['turn'],r['service'],r['slot'])]=r
assert sum(1 for d in data['dialogues'] for t in d['turns'] if t['input']['speaker']=='USER')==34022
# Every baseline missing target produces an opportunity, independently enumerated.
expected=set()
for d in ds:
 for i,t in enumerate(d['turns']):
  b=t['baseline']
  if b is None:continue
  for f in b['frames']:
   if f['service'] not in b['active_services']:continue
   for slot in data['targets'][f['service']]:
    if slot not in f['state']['slot_values']:expected.add((d['id'],i,f['service'],slot))
assert expected==rows.keys()
GRIDS=list(itertools.product([0,2],[2,4,8,16],[2,4]))
def select(r,g):
 source,score,gap=g;cs=r['candidates'];eligible=[i for i,c in enumerate(cs) if c['entity']['confidence']>=source]
 if not eligible:return None
 k=max(eligible,key=lambda i:cs[i]['score']);b=cs[k];bs=b['score']
 other=max([0.]+[c['score'] for c in cs if c['entity']['value'].lower()!=b['entity']['value'].lower()])
 return k if bs>=score and bs-other>=gap else None
# Verify every native grid decision, including every abstention.
num=0
for line in open(A/'decisions.jsonl'):
 z=json.loads(line);r=rows[(z['id'],z['turn'],z['service'],z['slot'])]
 assert select(r,z['grid'])==z['chosen'];num+=1
assert num==16*len(rows)
def outcome(t,p):
 fs={f['service']:f for f in p['frames']}; gold={f['service']:f for f in t['gold']}; assert gold.keys()<=schemas.keys()
 frames={s:bool(metrics.get_average_and_joint_goal_accuracy(g,fs[s],schemas[s],False)['joint_goal_accuracy']) for s,g in gold.items()}
 intent=all(metrics.get_active_intent_accuracy(g,fs[s]) for s,g in gold.items())
 return frames, bool(set(p['active_services'])==gold.keys() and all(frames.values()) and intent)
baseline={}; basecount=collections.Counter()
for d in ds:
 for i,t in enumerate(d['turns']):
  if t['baseline'] is None:continue
  fs,j=outcome(t,t['baseline']);baseline[(d['id'],i)]=(fs,j)
  basecount['user_turns']+=1;basecount['gold_frames']+=len(fs);basecount['joint_correct']+=j;basecount['strict_frames']+=sum(fs.values())
save('cal-baseline.json',basecount)
results=[]
for grid in GRIDS:
 tag='-'.join(map(str,grid));c=collections.Counter();predictions=[];changes=[];events=[]
 for d in ds:
  held={};out=[]
  for i,t in enumerate(d['turns']):
   if t['baseline'] is None:continue
   b=t['baseline'];p=copy.deepcopy(b);bf={f['service']:f for f in b['frames']};pf={f['service']:f for f in p['frames']};gold={f['service']:f for f in t['gold']}
   for service,f in bf.items():
    for slot in f['state']['slot_values']:held.pop((service,slot),None)
   for service in b['active_services']:
    for slot in data['targets'][service]:
     key=(service,slot)
     if slot in bf[service]['state']['slot_values'] or key in held:continue
     r=rows[(d['id'],i,service,slot)];idx=select(r,grid)
     if idx is not None:
      e=r['candidates'][idx]['entity'];held[key]={'value':e['value'],'source_turn':e['source_turn'],'start':e['start'],'end':e['end'],'role':e['role'],'binding_turn':i,'candidate_index':idx}
      events.append({'id':d['id'],'turn':i,'service':service,'slot':slot,'provenance':held[key]})
   for (service,slot),entry in held.items():
    assert slot not in pf[service]['state']['slot_values']
    pf[service]['state']['slot_values'][slot]=[entry['value']]
   assert p['active_services']==b['active_services']
   for service,f in pf.items():
    assert f['state']['active_intent']==bf[service]['state']['active_intent']
    for slot,vs in f['state']['slot_values'].items():
     old=bf[service]['state']['slot_values'].get(slot)
     if old==vs:continue
     assert old is None
     scored=service in gold and slot in {s['name'] for s in schemas[service]['slots']}
     gv=gold.get(service,{}).get('state',{}).get('slot_values',{}).get(slot,[])
     correct=scored and vs[0] in gv
     c['changed_slot_turns']+=1;c['scored_changes']+=scored;c['held_unscored']+=not scored
     c['predicted_inactive_changes']+=service not in p['active_services']
     c['new_correct_fills']+=correct and service in p['active_services'];c['scored_correct_inactive_no_credit']+=correct and service not in p['active_services'];c['new_wrong_assertions']+=scored and not correct
     changes.append({'id':d['id'],'turn':i,'service':service,'slot':slot,'baseline':old,'candidate':vs,'gold':gv,'scored':scored,'correct':correct,'predicted_active':service in p['active_services'],'provenance':held[(service,slot)]})
   fs,j=outcome(t,p);bfs,bj=baseline[(d['id'],i)]
   c['user_turns']+=1;c['gold_frames']+=len(fs);c['joint_correct']+=j;c['strict_frames']+=sum(fs.values())
   c['lost_joint']+=bj and not j;c['gained_joint']+=j and not bj
   c['lost_frames']+=sum(bfs[s] and not fs[s] for s in bfs);c['gained_frames']+=sum(fs[s] and not bfs[s] for s in bfs)
   c['inherited_wrong_joint_retained']+=not bj and not j
   c['inherited_wrong_frames_retained']+=sum(not bfs[s] and not fs[s] for s in bfs)
   out.append(p)
  predictions.append(out)
 assert c['user_turns']==basecount['user_turns'] and c['gold_frames']==basecount['gold_frames']
 passed=c['new_correct_fills']>=30 and c['new_wrong_assertions']==0 and c['lost_joint']==0 and c['lost_frames']==0
 result={'grid':grid,'passes':passed,**c};results.append(result)
 save(f'cal-{tag}-predictions.json',predictions);save(f'cal-{tag}-changes.json',changes);save(f'cal-{tag}-events.json',events)
 print(json.dumps(result),flush=True)
passed=[r for r in results if r['passes']];passed.sort(key=lambda r:(-r['new_correct_fills'],-r['gained_joint'],r['grid']))
save('cal-report.json',{'status':'CAL_PASS' if passed else 'REJECTED','selected':passed[0]['grid'] if passed else None,'baseline':basecount,'source_classification':source,'opportunities':total,'native_decisions_independently_verified':num,'grid':results,'limitation':'Empirical new-component validation, not unseen calibration; NONE includes absent/incomplete annotations.'})
