#!/usr/bin/env python3
"""Pinned train/dev-only cache and preregistered complete three-service selection."""
import concurrent.futures, hashlib, json, os, pathlib, urllib.request
ROOT=pathlib.Path(os.environ.get('SGD_ROOT', '/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full'))
OUT=ROOT/'selected'
VERSION='e852981ae34990f4358979625854259302feaa78'
SERVICES={'RideSharing_1','Travel_1','Weather_1'}
SOURCE=pathlib.Path('/home/gsc/.cache/glassmind/data/sgd/manifest.json')
manifest=json.loads(SOURCE.read_text())
assert sum(x['bytes'] for x in manifest['files']) < 600_000_000

def fetch(entry):
 path=entry['path']
 assert path=='LICENSE.txt' or path.split('/')[0] in ('train','dev')
 f=ROOT/path
 if not f.exists():
  url=f'https://raw.githubusercontent.com/{manifest["repository"]}/{VERSION}/{path}'
  with urllib.request.urlopen(url,timeout=60) as r:
   data=r.read(entry['bytes']+1)
  assert len(data)==entry['bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256']
  f.parent.mkdir(parents=True,exist_ok=True); f.write_bytes(data)
 data=f.read_bytes()
 assert len(data)==entry['bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256']
 return entry

OUT.mkdir(parents=True,exist_ok=True)
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
 files=list(pool.map(fetch,manifest['files']))
report={'version':VERSION,'license':'CC-BY-SA-4.0','files':files,'source_bytes':sum(e['bytes'] for e in files),'services':sorted(SERVICES),'splits':{}}
for split,n in [('train',127),('dev',20)]:
 schemas=[s for s in json.loads((ROOT/split/'schema.json').read_text()) if s['service_name'] in SERVICES]
 (OUT/f'{split}-schema.json').write_text(json.dumps(schemas))
 chosen=[]; total=0; counts={}
 for i in range(1,n+1):
  ds=json.loads((ROOT/split/f'dialogues_{i:03}.json').read_text()); total+=len(ds)
  for d in ds:
   for s in d['services']: counts[s]=counts.get(s,0)+1
   ss=set(d['services'])
   if (ss & SERVICES if split=='train' else ss and ss<=SERVICES): chosen.append(d)
 chosen.sort(key=lambda d:d['dialogue_id'])
 (OUT/f'{split}.json').write_text(json.dumps(chosen))
 report['splits'][split]={'all_dialogues':total,'train_service_counts':counts if split=='train' else None,'selected_dialogues':len(chosen),'single':sum(len(d['services'])==1 for d in chosen),'multi':sum(len(d['services'])>1 for d in chosen),'user_turns':sum(t['speaker']=='USER' for d in chosen for t in d['turns']),'selected_gold_frames':sum(f['service'] in SERVICES for d in chosen for t in d['turns'] if t['speaker']=='USER' for f in t['frames']),'ids':[d['dialogue_id'] for d in chosen]}
report['sample_ids']=report['splits']['dev']['ids'][:10]
report['multi_sample_ids']=[d['dialogue_id'] for d in chosen if len(d['services'])>1][:10]
(OUT/'manifest.json').write_text(json.dumps(report,indent=2))
print(json.dumps({k:v for k,v in report.items() if k!='files'},indent=2))
