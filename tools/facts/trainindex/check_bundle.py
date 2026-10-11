#!/usr/bin/env python3
"""Offline relocation, asset-binding and independent factual probes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run',type=Path,required=True)
    p.add_argument('--binary',type=Path,required=True)
    a=p.parse_args()
    a.run=a.run.resolve()
    a.binary=a.binary.resolve()
    source=a.run/'bundle1000'
    relocated=a.run/'relocated/bundle'
    relocated.parent.mkdir(exist_ok=True)
    shutil.copytree(source,relocated,dirs_exist_ok=True)
    probe_path=a.run/'probes.json'
    if not probe_path.exists():
        entities=json.loads((source/'index.json').read_bytes())['entities']
        specs=[('Q102642','P19','Where was Diane Keaton born?'),
               ('Q103917','P19','What city was Steven Soderbergh born in?'),
               ('Q8743','P27','What country was Thomas Edison a citizen of?'),
               ('Q1171','P36','What is the capital of Goa?'),
               ('Q83158','P20','Where did Jean Cocteau die?')]
        probes=[dict(question=text,subject=q,property=p,expected=entities[q]['claims'][p],source='actual independently fetched Wikidata claim, not validation annotations') for q,p,text in specs]
        probe_path.write_text(json.dumps(probes,indent=2)+'\n')
    probes=json.loads(probe_path.read_bytes())
    text=''.join('/reset\n'+p['question']+'\n' for p in probes)
    def invoke(path,stdin=text):
        return subprocess.run([str(a.binary),'talk','--json','--facts-dir',str(path)],input=stdin,text=True,capture_output=True,timeout=60,cwd=relocated.parent)
    original=invoke(source)
    moved=invoke(Path('bundle'))
    assert original.returncode==moved.returncode==0,(original.stderr,moved.stderr)
    assert original.stdout==moved.stdout
    replies=[json.loads(x) for x in moved.stdout.splitlines()][1::2]
    for probe,reply in zip(probes,replies,strict=True):
        probe['reply']=reply
        probe['correct']=reply.get('answers',[])==probe['expected']
    checks={'relocated_byte_identical':True}
    # Reuse only our copy, restoring bytes after each intentional corruption.
    for name,mutation,file in [
        ('index_asset_binding',lambda b:b+b' ','index.json'),
        ('model_asset_binding',lambda b:b+b' ','relation.json'),
        ('unsupported_manifest_schema',lambda b:json.dumps(dict(json.loads(b),version=999)).encode(),'manifest.json'),
        ('labels_asset_binding',lambda b:json.dumps(dict(json.loads(b),index_sha256='0'*64)).encode(),'labels.json'),
    ]:
        path=relocated/file
        saved=path.read_bytes()
        path.write_bytes(mutation(saved))
        try:
            result=invoke(Path('bundle'),'Where was Diane Keaton born?\n')
            checks[name]={'rejected':result.returncode!=0,'stderr':result.stderr}
            assert result.returncode!=0,name
        finally:
            path.write_bytes(saved)
    report=dict(checks=checks,probes=probes,correct=sum(p['correct'] for p in probes),total=len(probes),note='independent factual probes, not benchmark score; prompts frozen before dev read')
    (a.run/'bundle-checks.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    main()
