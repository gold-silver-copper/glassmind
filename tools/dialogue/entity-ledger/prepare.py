#!/usr/bin/env python3
"""Offline TRAIN-only supervision; runtime input remains InputTurn."""
import argparse, json, pathlib, hashlib
p=argparse.ArgumentParser();p.add_argument('--source',required=True,type=pathlib.Path);p.add_argument('--artifacts',required=True,type=pathlib.Path);a=p.parse_args()
ds=json.loads((a.source/'train.json').read_text()); schemas=json.loads((a.source/'full-train-schema.json').read_text());selected=json.loads((a.source/'schema.json').read_text());base=json.loads((a.artifacts/'baseline-train.json').read_text())
roles={s['service_name']+'/'+x['name'] for s in schemas for x in s['slots'] if not x['is_categorical']}
targets={s['service_name']:[x['name'] for x in s['slots'] if not x['is_categorical'] and any(x['name'] in it['required_slots'] or x['name'] in it['optional_slots'] for it in s['intents'])] for s in selected}
rows=[];counts={'FIT':0,'CAL':0,'user':0,'system':0,'unselected_user':0,'annotations':0}
assert len(ds)==len(base)==3174
for d,bs in zip(ds,base):
 cal=int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0;counts['CAL' if cal else 'FIT']+=1;by={s['turn_index']:s for s in bs};turns=[]
 for i,t in enumerate(d['turns']):
  spans=[];text=t['utterance'];gold=[];baseline=None
  if t['speaker']=='SYSTEM':
   counts['system']+=1
   for f in t.get('frames',[]):
    for sp in f.get('slots',[]):
     role=f['service']+'/'+sp['slot']
     if role not in roles:continue
     start,end=sp['start'],sp['exclusive_end'];assert 0<=start<end<=len(text)
     spans.append({'start':len(text[:start].encode()),'end':len(text[:end].encode()),'role':role});counts['annotations']+=1
  else:
   counts['user']+=1;counts['unselected_user']+=bool(any(f['service'] not in targets for f in t['frames']))
   baseline=by.pop(i);baseline={k:baseline[k] for k in ('turn_index','active_services','frames')}
   gold=[{'service':f['service'],'state':f['state']} for f in t['frames'] if f.get('state')]
  turns.append({'input':{k:t[k] for k in ('speaker','utterance')},'spans':spans,'baseline':baseline,'gold':gold})
 assert not by
 rows.append({'id':d['dialogue_id'],'cal':cal,'turns':turns})
(a.artifacts/'supervision.json').write_text(json.dumps({'roles':sorted(roles),'targets':targets,'dialogues':rows},separators=(',',':')))
(a.artifacts.parent/'reports/preparation.json').write_text(json.dumps({'counts':counts,'targets':targets,'note':'Only FIT supplies learned roles or weights; schema roles are validation metadata.'},indent=2))
print(counts)
