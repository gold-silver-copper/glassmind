#!/usr/bin/env python3
"""Independent reconstruction of all retained native linear scores and provenance."""
import collections, hashlib, json, pathlib, sys
import numpy as np
H=pathlib.Path(__file__).resolve().parent; A=H/'artifacts'
def load(p):return json.loads(p.read_text())
def fnv(s):
 h=2166136261
 for b in s.encode(): h=((h^b)*16777619)&0xffffffff
 return h
m=load(A/'model.json');d=load(A/'dataset.json');by_id={x['id']:x for x in d['dialogues']}
rw=np.array(m['role']['w'],dtype=np.float32);kw=np.array(m['rank']['w'],dtype=np.float32)
labels=np.arange(len(m['roles']),dtype=np.uint64)*np.uint64(2654435761)
c=collections.Counter();freq=collections.Counter();source_owner=collections.Counter()
for line in open(A/'source-spans.jsonl'):
 z=json.loads(line);r=z['row'];conv=by_id[r['id']];t=conv['turns'][r['turn']];c['source_rows']+=1
 assert r['source']==t['input']['utterance'] and t['input']['speaker']=='SYSTEM' and r['cal']==conv['cal']
 previous=next((x['input']['utterance'] for x in reversed(conv['turns'][:r['turn']]) if x['input']['speaker']=='USER'),'')
 assert previous==r['previous_user']
 assert sorted({fnv(x) for x in r['context']})==r['features']
 raw=r['raw'];value=r['source'].encode()[raw['start']:raw['end']].decode();assert value
 exact=sorted({sp['role'] for sp in t['spans'] if (sp['start'],sp['end'])==(raw['start'],raw['end'])});assert exact==r['exact_roles']
 if not r['cal'] and len(exact)==1:freq[exact[0]]+=1
 ix=(np.array(r['features'],dtype=np.uint64)[None,:]^labels[:,None])&np.uint64(262143)
 scores=np.cumsum(rw[ix],axis=1,dtype=np.float32)[:,-1]
 assert np.allclose(scores,z['scores'],rtol=0,atol=0.001),(r['id'],r['turn'])
 assert int(np.argmax(scores))==z['prediction']
 if z['prediction']:
  role=m['roles'][z['prediction']];source_owner[role.split('/')[0]]+=1
c['all_source_class_scores']=c['source_rows']*len(m['roles'])
expected=['NONE']+[k for k,n in sorted(freq.items(),key=lambda x:(-x[1],x[0])) if n>=20][:127];assert m['roles']==expected
for line in open(A/'proposals.jsonl'):
 r=json.loads(line);c['opportunities']+=1
 for cand in r['candidates']:
  e=cand['entity'];t=by_id[r['id']]['turns'][e['source_turn']]
  assert t['input']['speaker']=='SYSTEM' and e['source']==t['input']['utterance'] and e['value']==e['source'].encode()[e['start']:e['end']].decode()
  assert e['role']!='NONE'; assert e['source_turn']<r['turn']
  fs=np.array(cand['features'],dtype=np.uint64)&np.uint64(262143)
  score=float(np.cumsum(kw[fs],dtype=np.float32)[-1])
  assert abs(score-cand['score'])<0.001
  c['rank_scores']+=1
# Explicit baseline errors for every CAL selected-service scored slot, with old values retained.
sys.path.insert(0,str(H.parents[2]/'crates/dialogue/tools'))
import upstream_metrics as metrics
schemas={s['service_name']:s for s in load(pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/entityledger-source/schema.json'))}
errors=[]
for conv in d['dialogues']:
 if not conv['cal']:continue
 for i,t in enumerate(conv['turns']):
  if t['baseline'] is None:continue
  pred={f['service']:f for f in t['baseline']['frames']}
  for g in t['gold']:
   service=g['service']
   if service not in schemas:continue
   correctness,_,_=metrics.compare_slot_values(g['state']['slot_values'],pred[service]['state']['slot_values'],schemas[service],False)
   for slot,ok in zip(schemas[service]['slots'],correctness):
    if not ok:errors.append({'id':conv['id'],'turn':i,'service':service,'slot':slot['name'],'baseline':pred[service]['state']['slot_values'].get(slot['name']),'gold':g['state']['slot_values'].get(slot['name'])})
(A/'baseline-slot-errors.json').write_text(json.dumps(errors,indent=2)+'\n')
c['baseline_CAL_slot_errors']=len(errors)
report={'checks':c,'source_role_predictions_by_owner':source_owner,'all_source_scores_and_all_rank_scores_reconstructed':True,'all_raw_source_and_candidate_provenance_verified':True,'FIT_label_selection_independently_reconstructed':True}
(A/'reconstruction.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
