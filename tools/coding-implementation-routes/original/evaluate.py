#!/usr/bin/env python3
"""Run frozen public authored checks; no benchmark assets or expected patches."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess, time
from fixtures import CASES, MANIFEST, source
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--out',required=True);p.add_argument('--sample',action='store_true'); a=p.parse_args()
out=pathlib.Path(a.out).resolve();out.mkdir(parents=True,exist_ok=False)
results=[]
def snap(root):
 return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file() and 'target' not in p.parts}
for name,code,tests,target,accept in (CASES[:3] if a.sample else CASES):
 root=out/name; (root/'src').mkdir(parents=True)
 (root/'Cargo.toml').write_text(MANIFEST); (root/'src/lib.rs').write_text(source(code,tests));before=snap(root)
 env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
 start=time.monotonic()
 r=subprocess.run([a.binary,'do',str(root),'implement '+target],capture_output=True,text=True,env=env,timeout=100)
 (out/(name+'.log')).write_text(r.stdout+r.stderr)
 accepted='Applied test-verified proposal' in r.stdout
 after=snap(root)
 ok=(accepted==accept)
 if not accepted: ok=ok and after==before
 else:
  # Independent preservation check: every byte except placeholder text survives.
  changed=(root/'src/lib.rs').read_text();original=source(code,tests)
  marker='unimplemented!()' if 'unimplemented!()' in code else 'todo!()'
  left,right=original.split(marker,1)
  ok=ok and changed.startswith(left) and changed.endswith(right)
  check=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','30','cargo','test','--offline','-j1','--','--test-threads','4']
  verify=subprocess.run(check,cwd=root,env=env,capture_output=True,text=True,timeout=40)
  (out/(name+'-verify.log')).write_text(verify.stdout+verify.stderr)
  ok=ok and verify.returncode==0 and '1 passed' in verify.stdout
 results.append(dict(case=name,accepted=accepted,expected_accept=accept,ok=ok,exit=r.returncode,seconds=round(time.monotonic()-start,3),before=before,after=after))
 print(json.dumps(results[-1]),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
raise SystemExit(0 if all(r['ok'] for r in results) else 1)
