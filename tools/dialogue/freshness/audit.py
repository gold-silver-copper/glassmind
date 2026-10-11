#!/usr/bin/env python3
"""Exhaustive post-inference changed assertion audit, never runtime input."""
import argparse,collections,json,pathlib
p=argparse.ArgumentParser()
for n in ('data','baseline','candidate','inherited','output'):p.add_argument('--'+n,type=pathlib.Path,required=True)
a=p.parse_args();ds=sorted(json.loads(a.data.read_text()),key=lambda d:d['dialogue_id']);bs=json.loads(a.baseline.read_text());cs=json.loads(a.candidate.read_text());assert len(ds)==len(bs)==len(cs)
changes=[];counts=collections.Counter();by_key={}
for d,b,c in zip(ds,bs,cs):
 assert len(b)==len(c)
 for x,y in zip(b,c):
  i=x['turn_index'];assert i==y['turn_index'];assert x['active_services']==y['active_services']
  old={f['service']:f['state'] for f in x['frames']};new={f['service']:f['state'] for f in y['frames']};assert old.keys()==new.keys()
  gold={f['service']:f['state']['slot_values'] for f in d['turns'][i]['frames']}
  for service in new:
   assert old[service]['active_intent']==new[service]['active_intent']
   os=old[service]['slot_values'];ns=new[service]['slot_values']
   for slot in os.keys()|ns.keys():
    g=gold.get(service,{}).get(slot,[])
    def status(v):
     if service not in gold:return 'unscored-held'
     if not v:return 'absent'
     return 'correct' if all(z.lower() in [w.lower() for w in g] for z in v) else 'wrong'
    row=dict(dialogue_id=d['dialogue_id'],turn_index=i,service=service,slot=slot,before=os.get(slot),after=ns.get(slot),gold=g,before_status=status(os.get(slot)),status=status(ns.get(slot)))
    by_key[d['dialogue_id'],i,service,slot]=row
    if os.get(slot)==ns.get(slot):continue
    # All literal source matches are evidence of grounding only, not intended referent.
    row['source_matches']=[dict(turn_index=j,start=len(t['utterance'][:k].encode()),end=len(t['utterance'][:k+len(v)].encode()),value=v,utterance=t['utterance']) for j,t in enumerate(d['turns'][:i+1]) for v in ns.get(slot,[]) for k in range(len(t['utterance'])) if t['utterance'].startswith(v,k)]
    changes.append(row);counts[row['before_status']+'->'+row['status']]+=1
inherited=[]
for r in json.loads(a.inherited.read_text())['inherited']['claims']:
 row=by_key[r['dialogue_id'],r['turn_index'],r['service'],r['slot']];assert row['before']==r['after'];inherited.append(row)
out=dict(counts=counts,all_changes=changes,new_wrong_claims=[r for r in changes if r['status']=='wrong'],held_unscored=[r for r in changes if r['status']=='unscored-held'],inherited_claims=inherited,inherited_baseline_wrong=sum(r['before_status']=='wrong' for r in inherited),inherited_candidate_wrong=sum(r['status']=='wrong' for r in inherited),routing_intents_unchanged=True)
a.output.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if not isinstance(v,list)},indent=2))
