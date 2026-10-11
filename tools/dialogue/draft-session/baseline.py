#!/usr/bin/env python3
"""Verify supplied read-only assets and reproduce old-native prediction baseline."""
import argparse
import hashlib
import json
import pathlib
import subprocess
import time
p=argparse.ArgumentParser()
p.add_argument('--source',type=pathlib.Path,required=True)
p.add_argument('--run',type=pathlib.Path,required=True)
a=p.parse_args();a.run.mkdir(parents=True,exist_ok=True)
manifest=json.loads((a.source/'manifest.json').read_bytes())
assert manifest['source_revision']=='e852981ae34990f4358979625854259302feaa78'
assert manifest['license']=='CC-BY-SA-4.0'
assert manifest['model_sha256']=='fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88'
checks=[]
for item in manifest['files']:
    assert pathlib.Path(item['file']).name==item['file']
    data=(a.source/item['file']).read_bytes()
    digest=hashlib.sha256(data).hexdigest()
    assert digest==item['sha256'] and len(data)==item['bytes']
    checks.append(dict(file=item['file'],bytes=len(data),sha256=digest))
(a.run/'source-checks.json').write_text(json.dumps(checks,indent=2))
inputs=json.loads((a.source/'inputs.json').read_bytes())
assert len(inputs)==179 and sum(t['speaker']=='USER' for c in inputs for t in c)==1119
assert all(set(t)=={'speaker','utterance'} for c in inputs for t in c)
expected_bytes=(a.source/'baseline-predictions.json').read_bytes()
assert hashlib.sha256(expected_bytes).hexdigest()=='8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d'
expected=json.loads(expected_bytes)
(a.run/'first10-input.json').write_text(json.dumps(inputs[:10]))
reports=[]
for name,path,predictions in [('first10',a.run/'first10-input.json',expected[:10]),('full179',a.source/'inputs.json',expected)]:
    output=a.run/(name+'.json')
    command=['timeout','--kill-after=10s','600',str(a.source/'dialogue-multi'),'predict','--input',str(path),'--schema',str(a.source/'schema.json'),'--model',str(a.source/'model.json'),'--output',str(output)]
    start=time.monotonic();subprocess.run(command,check=True);elapsed=time.monotonic()-start
    data=output.read_bytes();assert json.loads(data)==predictions
    if name=='full179':assert data==expected_bytes
    reports.append(dict(name=name,command=command,seconds=elapsed,user_predictions=sum(map(len,predictions)),sha256=hashlib.sha256(data).hexdigest(),parity=True))
(a.run/'report.json').write_text(json.dumps(reports,indent=2))
print(json.dumps(reports,indent=2))
