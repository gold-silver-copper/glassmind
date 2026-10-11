"""Post-inference exhaustive assertion audit. Does not affect official scoring."""
import argparse,collections,json,pathlib
p=argparse.ArgumentParser()
for x in ('data','baseline','candidate','inherited','output'):p.add_argument('--'+x,type=pathlib.Path,required=True)
a=p.parse_args();ds=sorted(json.loads(a.data.read_text()),key=lambda d:d['dialogue_id']);bs=json.loads(a.baseline.read_text());cs=json.loads(a.candidate.read_text());assert len(ds)==len(bs)==len(cs)
changes=[];events=[];counts=collections.Counter();by_key={};route=True;intent=True;offers=True
for d,b,c in zip(ds,bs,cs):
 assert len(b)==len(c)
 for x,y in zip(b,c):
  i=y['turn_index'];assert x['turn_index']==i;u=d['turns'][i]['utterance']
  route &= x['active_services']==y['active_services'];offers &= x.get('offer_proposals',[])==y.get('offer_proposals',[])
  old={f['service']:f['state'] for f in x['frames']};new={f['service']:f['state'] for f in y['frames']};assert old.keys()==new.keys()
  gold={f['service']:f['state']['slot_values'] for f in d['turns'][i]['frames']}
  for e in y.get('sequence_proposals',[]):
   assert u.encode()[e['start']:e['end']].decode()==e['value'];assert u[e['char_start']:e['char_end']]==e['value']
   g=gold.get(e['service'],{}).get(e['slot'],[])
   events.append(dict(e,dialogue_id=d['dialogue_id'],turn_index=i,utterance=u,gold=g,status='held-unscored' if e['service'] not in gold else 'correct' if e['value'].lower() in [v.lower() for v in g] else 'wrong',before=old[e['service']]['slot_values'].get(e['slot']),after=new[e['service']]['slot_values'].get(e['slot'])))
  for service in new:
   intent &= old[service]['active_intent']==new[service]['active_intent'];os=old[service]['slot_values'];ns=new[service]['slot_values']
   for slot in sorted(set(os)|set(ns)):
    g=gold.get(service,{}).get(slot,[])
    def status(v):
     if service not in gold:return 'held-unscored'
     if not v:return 'absent'
     return 'correct' if all(z.lower() in [v.lower() for v in g] for z in v) else 'wrong'
    row=dict(dialogue_id=d['dialogue_id'],turn_index=i,service=service,slot=slot,before=os.get(slot),after=ns.get(slot),gold=g,before_status=status(os.get(slot)),status=status(ns.get(slot)),utterance=u)
    by_key[d['dialogue_id'],i,service,slot]=row
    if row['before']!=row['after']:changes.append(row);counts[row['before_status']+'->'+row['status']]+=1
inherited=[]
for r in json.loads(a.inherited.read_text())['inherited']['claims']:
 key=(r['dialogue_id'],r['turn_index'],r['service'],r['slot']);row=by_key[key];assert row['before']==r['after'];inherited.append(row)
report=dict(counts=counts,all_changes=changes,new_wrong=[r for r in changes if r['status']=='wrong'],correct_losses=[r for r in changes if r['before_status']=='correct' and r['status']!='correct'],new_correct=[r for r in changes if r['status']=='correct' and r['before_status']!='correct'],held_unscored=[r for r in changes if r['status']=='held-unscored'],inherited=inherited,inherited_seven=[r for r in inherited if r['before_status']=='wrong'],proposal_events=events,proposal_status=collections.Counter(r['status'] for r in events),routing_unchanged=route,intents_unchanged=intent,offers_unchanged=offers)
a.output.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n');print(json.dumps({k:v for k,v in report.items() if not isinstance(v,list)},indent=2))
