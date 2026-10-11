"""Check retained evidence; no scoring definitions or runtime source inputs."""
import argparse,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);p.add_argument('--label',default='guarded');a=p.parse_args();r=a.run
read=lambda path:json.loads(path.read_text())
b=read(r/'baseline-full/report.json');c=read(r/(a.label+'-full')/'report.json');by={d['source']:d for d in c['details']}
assert b['inputs_sha256']==c['inputs_sha256']
retained=[];gains=[]
for d in b['details']:
 n=by[d['source']]
 if d['verdict']=='verified':
  assert n==d,(d,n)
  retained.append(d['source'])
 elif n['verdict']=='verified':gains.append(n)
assert len(retained)==6 and len(gains)>=1
assert not c['totals'].get('failed_verification',0)
baseline_sample=read(r/'baseline-sample/report.json');sample=read(r/(a.label+'-sample')/'report.json');assert sample==baseline_sample
native=read(r/('authored-'+a.label)/'report.json');supplement=read(r/('supplement-'+a.label)/'report.json')
assert len(native)==20 and len(supplement)==5
assert all(d['exact'] and d['original_unchanged'] for d in native+supplement)
assert all(d['verified'] for d in native+supplement if d['positive'])
manifest=read(r/'train-source-manifest.json');source=Path('/home/gsc/.cache/glassmind/sources')
assert all(hashlib.sha256((source/f).read_bytes()).hexdigest()==d['sha256'] for f,d in manifest.items())
source_files=[Path('crates/world-rust/src/argument_swap.rs'),Path('crates/world-rust/src/checks.rs'),Path('crates/world-rust/src/lib.rs'),Path('crates/world-rust/tests/argument_swap.rs')]
metadata={str(p):dict(sha256=hashlib.sha256(p.read_bytes()).hexdigest(),bytes=p.stat().st_size) for p in source_files}
summary=dict(baseline=b['totals'],candidate=c['totals'],sample=sample['totals'],old_patches_identical=retained,new_verified=gains,authored_positive_exact=sum(d['positive'] for d in native),authored_refusal_exact=sum(not d['positive'] for d in native),supplement_exact=len(supplement),originals_unchanged=True,train_source_files=len(manifest),train_source_bytes=sum(d['bytes'] for d in manifest.values()),train_source_hashes_preserved=True,source_identity=metadata,parent_full765_gate='pending; no worker score claim')
(r/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
