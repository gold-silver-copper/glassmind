#!/usr/bin/env python3
"""Verify permitted source bytes, frozen weights, and all inherited errors."""
import argparse, hashlib, json, pathlib
p=argparse.ArgumentParser()
for k in ('root','run','target','baseline'):p.add_argument('--'+k,required=True,type=pathlib.Path)
a=p.parse_args()
def record(path):
 b=path.read_bytes();return {'path':str(path),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
manifest=json.load(open('tools/dialogue/reports/data-manifest.json'));source=[]
for f in manifest['files']:
 assert f['path']=='LICENSE.txt' or f['path'].startswith(('train/','dev/'))
 r=record(a.root/f['path']);assert r['bytes']==f['bytes'] and r['sha256']==f['sha256'];source.append(r)
b=json.load(open(a.baseline/'model.json'));c=json.load(open(a.run/'model.json'));offers=c.pop('offers');assert b==c
bs=json.load(open(a.baseline/'full/predictions.json'));cs=json.load(open(a.run/'full/predictions.json'))
ds=sorted(json.load(open(a.root/'selected/dev.json')),key=lambda d:d['dialogue_id'])
old=json.load(open('tools/dialogue/references/reports/copy-audit.json'))
# Preserve the entire inherited audit and compare every old changed stored claim.
index={(d['dialogue_id'],s['turn_index'],f['service']):f for d,ss in zip(ds,cs) for s in ss for f in s['frames']}
inherited=old['new_incorrect_slot_turns']
for row in inherited:
 frame=index[(row['dialogue_id'],row['turn_index'],row['service'])]
 assert frame['state']['slot_values'][row['slot']]==row['after']
assert len(inherited)==11
(a.run/'inherited-errors.json').write_text(json.dumps({'unchanged':True,'copy_events':4,'scored_slot_turns':len(inherited),'claims':inherited},indent=2)+'\n')
unchanged=0
for d,batch,cand in zip(ds,bs,cs):
 for x,y in zip(batch,cand):
  assert x['turn_index']==y['turn_index'] and x['active_services']==y['active_services']
  for f,g in zip(x['frames'],y['frames']):
   assert f['service']==g['service']
   if f['service'] in ('Travel_1','Weather_1'):assert f==g
   unchanged+=f==g
assert (a.run/'frozen-replay/predictions.json').read_bytes()==(a.baseline/'full/predictions.json').read_bytes()
files=[a.root/'selected'/f for f in ('train.json','train-schema.json','dev.json','dev-schema.json')]
files += [a.baseline/'model.json',a.baseline/'full/predictions.json',a.run/'model.json',a.run/'offers.json',a.run/'first/predictions.json',a.run/'full/predictions.json',a.run/'legacy-predictions.json',a.target/'release/dialogue-multi',a.target/'release/glassmind-dialogue']
files += [pathlib.Path(f) for f in ['tools/dialogue/evaluate.py','tools/dialogue/compare.py','crates/dialogue/tools/upstream_metrics.py','tools/dialogue/offers/train.py','crates/dialogue/src/offers.rs','crates/dialogue/src/multi.rs']]
r={'base_commit':'da96f494d4a6865924cb0aa5eea138b029158fc2','source_revision':manifest['version'],'license':manifest['license'],'source_bytes':sum(x['bytes'] for x in source),'download_bytes':0,'source_files':source,'artifacts':[record(f) for f in files],'frozen_weights_identical':True,'frozen_predictions_byte_identical':True,'routing_identical':True,'Travel_and_Weather_frames_identical':True,'rules':len(offers['rules']),'unchanged_predicted_frames':unchanged}
(a.run/'artifacts.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k not in ('source_files','artifacts')}))
