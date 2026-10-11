#!/usr/bin/env python3
"""Complete CAL slot-turn audit using unchanged upstream strict metrics."""
import argparse, collections, copy, json, pathlib, sys, warnings, itertools
p=argparse.ArgumentParser();p.add_argument('--artifacts',type=pathlib.Path,required=True);p.add_argument('--schema',type=pathlib.Path,required=True);a=p.parse_args()
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[3]/'crates/dialogue/tools'))
with warnings.catch_warnings():
 warnings.simplefilter('ignore');import upstream_metrics as metrics
schemas={s['service_name']:s for s in json.loads(a.schema.read_text())};data=json.loads((a.artifacts/'supervision.json').read_text());ds=[d for d in data['dialogues'] if d['cal']]
proposals={};stats=collections.Counter();sourcecounts=collections.Counter()
for line in (a.artifacts/'proposals.jsonl').open():
 r=json.loads(line);stats[('CAL' if r['cal'] else 'FIT')+'_opportunities']+=1;stats[('CAL' if r['cal'] else 'FIT')+'_no_candidate']+=not r['candidates']
 for c in r['candidates']:
  e=c['entity'];assert e['source'].encode()[e['start']:e['end']].decode()==e['value'];assert e['source_turn']<r['turn'];assert isinstance(c['score'],(float,int))
  c.pop('features');sourcecounts[e['role']]+=1
 if r['cal']:proposals[(r['id'],r['turn'],r['service'],r['slot'])]=r
choices=collections.defaultdict(dict)
for line in (a.artifacts/'decisions.jsonl').open():
 r=json.loads(line);key=(r['id'],r['turn'],r['service'],r['slot']);choices[tuple(r['grid'])][key]=r['chosen']
assert len(choices)==16
reports=[];audit=(a.artifacts/'cal-changes.jsonl').open('w');basecorrect=None
for grid,decisions in sorted(choices.items()):
 c=collections.Counter();lost_joint=[];lost_frames=[];gained_joint=[];initial=[]
 for d in ds:
  overlay={}
  for ti,t in enumerate(d['turns']):
   if t['baseline'] is None:continue
   c['user_turns']+=1;b=t['baseline'];bf={f['service']:f for f in b['frames']};gold={f['service']:f for f in t['gold'] if f['service'] in schemas};cf=copy.deepcopy(bf)
   # Frozen state authority wins; held ledger fills persist until an old mechanism fills.
   for key in list(overlay):
    service,slot=key
    if slot in bf[service]['state']['slot_values']:del overlay[key]
   for service in b['active_services']:
    for slot in data['targets'][service]:
     key=(d['id'],ti,service,slot)
     if slot in bf[service]['state']['slot_values'] or (service,slot) in overlay:continue
     assert key in decisions
     ix=decisions[key]
     if ix is None:continue
     cand=proposals[key]['candidates'][ix];overlay[service,slot]=cand
     initial.append({'id':d['id'],'turn':ti,'service':service,'slot':slot,'candidate':cand})
   for (service,slot),cand in overlay.items():
    value=cand['entity']['value'];cf[service]['state']['slot_values'][slot]=[value]
    g=gold.get(service);gs=g['state']['slot_values'].get(slot,[]) if g else None
    correct=bool(gs is not None and metrics.noncat_slot_value_match(gs,value,False)==1)
    scored=g is not None
    kind='correct_scored' if correct else 'wrong_scored' if scored else 'held_unscored'
    c[kind]+=1;c['changed_slot_turns']+=1;c['inactive_changes']+=service not in b['active_services']
    audit.write(json.dumps({'grid':grid,'id':d['id'],'turn':ti,'service':service,'slot':slot,'baseline':None,'value':value,'gold':gs,'kind':kind,'predicted_active':service in b['active_services'],'evidence':cand})+'\n')
   route=set(b['active_services'])==set(gold);bj=cj=route
   for service,g in gold.items():
    c['gold_frames']+=1
    bs=metrics.get_average_and_joint_goal_accuracy(g,bf[service],schemas[service],False)['joint_goal_accuracy']==1
    cs=metrics.get_average_and_joint_goal_accuracy(g,cf[service],schemas[service],False)['joint_goal_accuracy']==1
    intent=metrics.get_active_intent_accuracy(g,bf[service])==1
    c['baseline_correct_frames']+=bs;c['candidate_correct_frames']+=cs
    if bs and not cs:lost_frames.append([d['id'],ti,service])
    bj=bj and bs and intent;cj=cj and cs and intent
   c['baseline_correct_joint']+=bj;c['candidate_correct_joint']+=cj
   if bj and not cj:lost_joint.append([d['id'],ti])
   if not bj and cj:gained_joint.append([d['id'],ti])
 passed=c['correct_scored']>=30 and c['wrong_scored']==0 and not lost_joint and not lost_frames
 reports.append({'grid':grid,'counts':{k:int(v) for k,v in c.items()},'initial_events':len(initial),'pass':passed,'lost_joint':lost_joint,'lost_frames':lost_frames,'gained_joint':gained_joint})
 (a.artifacts/f'cal-initial-{grid[0]}-{grid[1]}-{grid[2]}.json').write_text(json.dumps(initial))
audit.close();passed=[r for r in reports if r['pass']];selected=max(passed,key=lambda r:(r['counts']['correct_scored'],r['grid']))['grid'] if passed else None
report={'status':'PROVISIONAL_CAL_PASS_REQUIRES_INTEGRATED_REPLAY' if selected else 'REJECTED_CAL','selected':selected,'kind':'empirical new-component validation; frozen baseline saw all TRAIN','simulation':'shadow persistent overlay; no feedback into old references/offers; never sufficient for DEV authorization','grid':reports,'opportunities':dict(stats),'all_CAL_dialogues':len(ds),'all_CAL_user_turns':sum(t['baseline'] is not None for d in ds for t in d['turns'])}
(a.artifacts.parent/'reports/gate.json').write_text(json.dumps(report,indent=2))
for r in reports:print(r['grid'],r['counts'], 'PASS' if r['pass'] else 'FAIL')
print(report['status'])
