#!/usr/bin/env python3
"""Fit value-free cross-service relations using selected official TRAIN only.

Fixed rule: >=20 opportunities, no observed mismatches. Every active target
frame with an absent prior gold input slot is an opportunity, even if the gold
new state does not contain that slot. All sources come from PREVIOUS turns.
No service-name rules, entity vocabulary, utterance IDs or answers are saved.
"""
import argparse, collections, hashlib, json, pathlib
p=argparse.ArgumentParser()
p.add_argument('--train', required=True, type=pathlib.Path)
p.add_argument('--schema', required=True, type=pathlib.Path)
p.add_argument('--output', required=True, type=pathlib.Path)
a=p.parse_args()
schemas=json.loads(a.schema.read_text())
slots={s['service_name']: {q['name'] for q in s['slots'] if not q['is_categorical']} & {n for i in s['intents'] for n in i['required_slots']+list(i['optional_slots'])} for s in schemas}
counts=collections.defaultdict(lambda:[0,0])
for d in json.loads(a.train.read_text()):
    prior={}
    for t in d['turns']:
        if t['speaker']!='USER': continue
        for f in t['frames']:
            target=f['service']
            if target not in slots or 'state' not in f: continue
            gold=f['state']['slot_values']
            for slot in sorted(slots[target]):
                if slot in prior.get(target,{}): continue
                for source,state in sorted(prior.items()):
                    if source==target: continue
                    for source_slot,values in sorted(state.items()):
                        if source_slot not in slots[source] or len(values)!=1 or values[0]=='dontcare': continue
                        key=(source,source_slot,target,slot)
                        counts[key][1]+=1
                        counts[key][0]+=int(values[0].lower() in [v.lower() for v in gold.get(slot,[])])
        for f in t['frames']:
            if f['service'] in slots and 'state' in f:
                prior[f['service']]=f['state']['slot_values']
relations=[]; evidence=[]
for (source,source_slot,target,target_slot),(matches,opportunities) in sorted(counts.items()):
    row=dict(source_service=source,source_slot=source_slot,target_service=target,target_slot=target_slot,matches=matches,opportunities=opportunities)
    evidence.append(row)
    if matches==opportunities and matches>=20: relations.append(row)
model=dict(version=1,minimum_support=20,relations=relations)
a.output.write_text(json.dumps(model,indent=2)+'\n')
a.output.with_suffix('.evidence.json').write_text(json.dumps(dict(train_sha256=hashlib.sha256(a.train.read_bytes()).hexdigest(),schema_sha256=hashlib.sha256(a.schema.read_bytes()).hexdigest(),relations=evidence),indent=2)+'\n')
print(json.dumps(model,indent=2))
