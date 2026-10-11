#!/usr/bin/env python3
"""Offline annotations; runtime features are generated separately from InputTurn."""
import collections, hashlib, json, pathlib
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[3].parent
# Fixed supplied read-only packages; no old report discovery.
ROOT=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010')
S=ROOT/'entityledger-source'; C=ROOT/'contextledger-source'; OUT=HERE/'artifacts'
def read(p): return json.loads(p.read_text())
def save(n,v): (OUT/n).write_text(json.dumps(v,ensure_ascii=False,separators=(',',':'))+'\n')
verified={}
for p in [S,C]:
 m=read(p/'manifest.json')
 for n,r in m.items():
  b=(p/n).read_bytes();assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256']
 verified[p.name]={'files':len(m),'bytes':sum(r['bytes'] for r in m.values()),'manifest':m}
ds=sorted(read(S/'train.json'),key=lambda d:d['dialogue_id']); baseline=read(C/'baseline-train.json'); inputs=read(C/'train-input.json')
assert inputs==[[{k:t[k] for k in ('speaker','utterance')} for t in d['turns']] for d in ds]
assert len(ds)==len(baseline)==3174
schemas=read(S/'full-train-schema.json'); selected=read(S/'schema.json')
roles=sorted({s['service_name']+'/'+sl['name'] for s in schemas for sl in s['slots']})
targets={}
for s in selected:
 names={k for it in s['intents'] for k in list(it['required_slots'])+list(it['optional_slots'])}
 targets[s['service_name']]=sorted(sl['name'] for sl in s['slots'] if not sl['is_categorical'] and sl['name'] in names)
result=[]; counts=collections.Counter()
for d,ps in zip(ds,baseline):
 bi={p['turn_index']:p for p in ps}; assert len(bi)==len(ps)
 assert sorted(bi)==[i for i,t in enumerate(d['turns']) if t['speaker']=='USER']
 cal=int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0;counts['CAL' if cal else 'FIT']+=1
 turns=[]
 for i,t in enumerate(d['turns']):
  counts[t['speaker']]+=1;spans=[]
  for f in t['frames']:
   for sp in f.get('slots',[]):
    a,b=sp['start'],sp['exclusive_end'];assert 0<=a<b<=len(t['utterance'])
    role=f['service']+'/'+sp['slot']; assert role in roles
    spans.append({'start':len(t['utterance'][:a].encode()),'end':len(t['utterance'][:b].encode()),'role':role})
  turns.append({'input':{k:t[k] for k in ('speaker','utterance')},'spans':spans,'baseline':bi.get(i),'gold':[{'service':f['service'],'state':f['state']} for f in t['frames'] if 'state' in f]})
 result.append({'id':d['dialogue_id'],'cal':cal,'turns':turns})
assert counts['FIT']==2532 and counts['CAL']==642
save('dataset.json',{'roles':roles,'targets':targets,'dialogues':result})
save('source-verification.json',{'packages':verified,'alignment':'exact sorted utterance-only inputs and every USER index','counts':counts,'targets':targets,'license':'CC-BY-SA-4.0','revision':'e852981ae34990f4358979625854259302feaa78'})
print(json.dumps(counts));print(targets)
