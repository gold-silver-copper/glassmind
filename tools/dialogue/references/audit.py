#!/usr/bin/env python3
"""Post-inference diagnostics only; does not define or change acceptance metrics."""
import argparse, collections, hashlib, json, pathlib
p=argparse.ArgumentParser()
for name in ('data','baseline','candidate','model','output'):
    p.add_argument('--'+name,required=True,type=pathlib.Path)
a=p.parse_args()
ds=sorted(json.loads(a.data.read_text()),key=lambda d:d['dialogue_id'])
bs=json.loads(a.baseline.read_text());cs=json.loads(a.candidate.read_text())
model=json.loads(a.model.read_text());counts=collections.Counter();changes=[];events=[]
assert len(ds)==len(bs)==len(cs)
for d,b,c in zip(ds,bs,cs):
    previous={}
    for x,y in zip(b,c):
        assert x['turn_index']==y['turn_index']
        i=x['turn_index'];gold={f['service']:f['state']['slot_values'] for f in d['turns'][i]['frames']}
        bf={f['service']:f['state']['slot_values'] for f in x['frames']}
        current={f['service']:f['state']['slot_values'] for f in y['frames']}
        assert x['active_services']==y['active_services']
        for service,slots in current.items():
            for slot,values in slots.items():
                if values==bf[service].get(slot):continue
                assert slot not in bf[service], 'resolver changed an existing slot'
                exact=values[0] in gold[service].get(slot,[]) if service in gold else None
                category='correct_added' if exact else 'incorrect_added' if exact is False else 'held_unscored'
                counts[category]+=1
                row=dict(dialogue_id=d['dialogue_id'],turn_index=i,service=service,slot=slot,before=bf[service].get(slot),after=values,gold=gold.get(service,{}).get(slot),exact=exact)
                changes.append(row)
                if values!=previous.get(service,{}).get(slot):
                    sources=[]
                    for r in model['references']['relations']:
                        if r['target_service']==service and r['target_slot']==slot and previous.get(r['source_service'],{}).get(r['source_slot'])==values:
                            sources.append([r['source_service'],r['source_slot']])
                    assert sources, 'copy not grounded in prior predicted state'
                    occurrences=[j for j,t in enumerate(d['turns'][:i]) if values[0].lower() in t['utterance'].lower()]
                    assert occurrences, 'missing prior text support'
                    events.append(dict(**row,sources=sources,support_turns=occurrences))
        previous=current
out=dict(counts=dict(counts),copy_events=events,changed_slot_turns=changes,new_incorrect_slot_turns=[r for r in changes if r['exact'] is False],routing_unchanged=True)
a.output.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(dict(counts=out['counts'],copy_events=len(events),incorrect_copy_events=sum(r['exact'] is False for r in events)),indent=2))
