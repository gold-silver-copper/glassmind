#!/usr/bin/env python3
"""Exhaustive post-inference slot audit, separate from the unchanged scorer."""
import argparse, collections, json, pathlib
p=argparse.ArgumentParser()
for n in ('data','baseline','candidate','inherited','output'):p.add_argument('--'+n,type=pathlib.Path,required=True)
a=p.parse_args();ds=sorted(json.loads(a.data.read_text()),key=lambda d:d['dialogue_id']);bs=json.loads(a.baseline.read_text());cs=json.loads(a.candidate.read_text())
assert len(ds)==len(bs)==len(cs)
changes=[];events=[];counts=collections.Counter();by_key={};origins={}
for d,b,c in zip(ds,bs,cs):
    assert len(b)==len(c)
    for x,y in zip(b,c):
        i=y['turn_index'];assert x['turn_index']==i
        assert x['active_services']==y['active_services']
        assert x.get('offer_proposals',[])==y.get('offer_proposals',[])
        old={f['service']:f['state'] for f in x['frames']};new={f['service']:f['state'] for f in y['frames']}
        gold={f['service']:f['state']['slot_values'] for f in d['turns'][i]['frames']}
        assert old.keys()==new.keys()
        for event in y.get('boundary_proposals',[]):
            source=d['turns'][event['source_turn']]['utterance']
            assert event['source_turn']<i
            assert source.encode()[event['start']:event['end']].decode()==event['value']
            assert event['value'].startswith(event['original'])
            event=dict(event,dialogue_id=d['dialogue_id'],turn_index=i,source_utterance=source)
            events.append(event);origins[d['dialogue_id'],event['service'],event['slot']]=event
        for service in new:
            assert old[service]['active_intent']==new[service]['active_intent']
            os=old[service]['slot_values'];ns=new[service]['slot_values']
            for slot in set(os)|set(ns):
                key=(d['dialogue_id'],i,service,slot);g=gold.get(service,{}).get(slot,[])
                status=lambda vs:'unscored-held' if service not in gold else 'correct' if vs and all(v.lower() in [z.lower() for z in g] for v in vs) else 'wrong'
                row=dict(dialogue_id=d['dialogue_id'],turn_index=i,service=service,slot=slot,before=os.get(slot),after=ns.get(slot),gold=g,before_status=status(os.get(slot)),status=status(ns.get(slot)),literal_exact=ns.get(slot)==g)
                by_key[key]=row
                if os.get(slot)==ns.get(slot):continue
                row=dict(row,origin=origins.get((d['dialogue_id'],service,slot)))
                assert row['origin'] is not None
                changes.append(row);counts[row['before_status']+'->'+row['status']]+=1
inherited=[]
for r in json.loads(a.inherited.read_text())['claims']:
    key=(r['dialogue_id'],r['turn_index'],r['service'],r['slot']);row=by_key[key]
    assert row['before']==r['after'];inherited.append(row)
report=dict(counts=counts,events=events,all_changes=changes,all_new_wrong_claims=[r for r in changes if r['status']=='wrong'],unscored_held=[r for r in changes if r['status']=='unscored-held'],inherited=dict(original_events=4,original_scored_slot_turns=11,corrected_scored_slot_turns=sum(r['status']=='correct' for r in inherited),remaining_wrong_scored_slot_turns=sum(r['status']=='wrong' for r in inherited),claims=inherited),routing_intents_offers_unchanged=True)
a.output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(counts=counts,events=len(events),new_wrong=len(report['all_new_wrong_claims']),inherited_corrected=report['inherited']['corrected_scored_slot_turns'],inherited_remaining=report['inherited']['remaining_wrong_scored_slot_turns']),indent=2))
