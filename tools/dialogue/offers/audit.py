#!/usr/bin/env python3
"""Post-inference exhaustive added-claim audit; does not select/model or score."""
import argparse, collections, json, pathlib
p=argparse.ArgumentParser()
for k in ('data','baseline','candidate','output'):p.add_argument('--'+k,required=True)
a=p.parse_args();ds=sorted(json.load(open(a.data)),key=lambda d:d['dialogue_id']);bs=json.load(open(a.baseline));cs=json.load(open(a.candidate))
assert len(ds)==len(bs)==len(cs)
counts=collections.Counter();changes=[];events=[];last_event={}
for d,b,c in zip(ds,bs,cs):
 for x,y in zip(b,c):
  i=y['turn_index'];assert x['turn_index']==i
  gold={f['service']:f['state']['slot_values'] for f in d['turns'][i]['frames']}
  old={f['service']:f['state']['slot_values'] for f in x['frames']}
  for event in y.get('offer_proposals',[]):
   source=d['turns'][event['source_turn']]
   assert source['speaker']=='SYSTEM' and event['source_turn']==i-1
   assert source['utterance'].encode()[event['start']:event['end']].decode()==event['value']
   event=dict(event,dialogue_id=d['dialogue_id'],turn_index=i,source_utterance=source['utterance'],user_utterance=d['turns'][i]['utterance'])
   events.append(event);last_event[d['dialogue_id'],event['service'],event['slot']]=event
  for f in y['frames']:
   s=f['service'];new=f['state']['slot_values']
   for slot in set(new)|set(old[s]):
    if old[s].get(slot)==new.get(slot):continue
    g=gold.get(s,{}).get(slot,[])
    status='unscored-held' if s not in gold else 'correct' if new.get(slot) and all(v.lower() in [z.lower() for z in g] for v in new[slot]) else 'wrong'
    row={'dialogue_id':d['dialogue_id'],'turn_index':i,'service':s,'slot':slot,'before':old[s].get(slot),'after':new.get(slot),'gold':g,'status':status,'origin':last_event.get((d['dialogue_id'],s,slot))}
    changes.append(row);counts[status]+=1
out={'counts':counts,'events':events,'all_changes':changes,'all_wrong_claims':[r for r in changes if r['status']=='wrong']}
pathlib.Path(a.output).write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'counts':counts,'events':len(events),'wrong_events':len({(r['dialogue_id'],r['origin']['turn_index'] if r['origin'] else None,r['slot']) for r in out['all_wrong_claims']})}))
