#!/usr/bin/env python3
"""Independent raw-decision/provenance review; no fit, tuning or DEV."""
import argparse, collections, hashlib, json, pathlib, math
p=argparse.ArgumentParser();p.add_argument('--artifacts',type=pathlib.Path,required=True);a=p.parse_args();data=json.loads((a.artifacts/'supervision.json').read_text());by={d['id']:d for d in data['dialogues']};model=json.loads((a.artifacts/'model.json').read_text());counts=collections.Counter();roles=collections.Counter()
for d in data['dialogues']:
 assert d['cal']==(int(hashlib.sha256(d['id'].encode()).hexdigest(),16)%5==0)
 if not d['cal']:
  for t in d['turns']:
   if t['input']['speaker']=='SYSTEM':roles.update(s['role'] for s in t['spans'])
expected=[r for r,n in sorted(roles.items(),key=lambda x:(-x[1],x[0])) if n>=20][:128];assert model['roles']==expected
for name in ['boundary','role','rank']:assert all(math.isfinite(x) for x in model[name]['w'])
assert (a.artifacts/'model.json').stat().st_size<=16*1024*1024
ps={};m=model['rank']['w'];maxerr=0.
for line in (a.artifacts/'proposals.jsonl').open():
 r=json.loads(line);d=by[r['id']];t=d['turns'][r['turn']];b=t['baseline'];assert b is not None and r['service'] in b['active_services']
 bf=next(f for f in b['frames'] if f['service']==r['service']);assert r['slot'] not in bf['state']['slot_values'];assert r['slot'] in data['targets'][r['service']]
 gold=next((f['state']['slot_values'].get(r['slot'],[]) for f in t['gold'] if f['service']==r['service']),[]);assert r['gold']==gold
 for c in r['candidates']:
  counts['candidates']+=1;e=c['entity'];s=d['turns'][e['source_turn']]['input'];assert s['speaker']=='SYSTEM' and s['utterance']==e['source'];assert e['source_turn']<r['turn'];assert e['source'].encode()[e['start']:e['end']].decode()==e['value']
  age=sum(t['input']['speaker']=='SYSTEM' for t in d['turns'][e['source_turn']+1:r['turn']]);assert age<32
  assert c['correct']==any(v.lower()==e['value'].lower() for v in gold)
  score=sum(m[f & (262144-1)] for f in c['features']);err=abs(c['score']-score);maxerr=max(maxerr,err);assert err<0.001
 counts['opportunities']+=1
 if r['cal']:ps[(r['id'],r['turn'],r['service'],r['slot'])]=r
for line in (a.artifacts/'decisions.jsonl').open():
 d=json.loads(line);r=ps[(d['id'],d['turn'],d['service'],d['slot'])];source,score,gap=d['grid'];cs=r['candidates'];eligible=[i for i,c in enumerate(cs) if c['entity']['confidence']>=source];choice=None
 if eligible:
  best=max(eligible,key=lambda i:cs[i]['score']);b=cs[best];alternative=max([0]+[c['score'] for c in cs if c['entity']['value'].lower()!=b['entity']['value'].lower()])
  if b['score']>=score and b['score']-alternative>=gap:choice=best
 assert choice==d['chosen'];counts['decisions']+=1
# Characterize the strictest declared grid after rejection only, not a new selection.
wrong=collections.Counter();inherited=collections.Counter(strict_existing_nonmatching_assertions=0);g=(2.,16.,4.);wrong_initial=[]
for line in (a.artifacts/'cal-changes.jsonl').open():
 r=json.loads(line)
 if tuple(r['grid'])!=g or r['kind']!='wrong_scored':continue
 e=r['evidence']['entity'];source_spans=by[r['id']]['turns'][e['source_turn']]['spans'];exact=[s for s in source_spans if s['start']==e['start'] and s['end']==e['end']];gs=r['gold'] or []
 cause='case_only' if any(v.lower()==r['value'].lower() for v in gs) else 'span_boundary' if not exact else 'role_error' if not any(s['role']==e['role'] for s in exact) else 'reference_selection'
 wrong[cause]+=1;wrong_initial.append({'id':r['id'],'turn':r['turn'],'service':r['service'],'slot':r['slot'],'cause':cause,'source_role':e['role'],'gold_roles':[s['role'] for s in exact],'value':r['value'],'gold':gs})
# Existing wrong assertions are never overwritten by the overlay; count explicitly.
for d in data['dialogues']:
 if not d['cal']:continue
 for t in d['turns']:
  if not t['baseline']:continue
  gold={f['service']:f for f in t['gold']}
  for f in t['baseline']['frames']:
   if f['service'] not in gold:continue
   for slot,values in f['state']['slot_values'].items():
    gv=gold[f['service']]['state']['slot_values'].get(slot,[])
    if values and values[0] not in gv:inherited['strict_existing_nonmatching_assertions']+=1
report={'verified':dict(counts),'FIT_role_labels':len(expected),'max_rank_reconstruction_error_f64_vs_f32':maxerr,'strictest_grid_post_rejection_wrong_scored':dict(wrong),'inherited_CAL_assertions_untouched':dict(inherited),'note':'Cause hierarchy is diagnostic only; annotation absence is not proof of incorrect entity. No threshold/model changed.'}
(a.artifacts.parent/'reports/independent-review.json').write_text(json.dumps(report,indent=2));(a.artifacts/'strictest-wrong-diagnosis.json').write_text(json.dumps(wrong_initial,indent=2));print(json.dumps(report,indent=2))
