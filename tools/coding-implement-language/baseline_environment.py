#!/usr/bin/env python3
import argparse,hashlib,json,os,pathlib,subprocess
from original.fixtures import MANIFEST,CASES,source
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--out',required=True);p.add_argument('--request',default='implement aggregate');p.add_argument('--no-tmp',action='store_true');a=p.parse_args()
out=pathlib.Path(a.out).resolve();out.mkdir(parents=True,exist_ok=False);root=out/'project';(root/'src').mkdir(parents=True)
(root/'Cargo.toml').write_text(MANIFEST);(root/'src/lib.rs').write_text(source(CASES[0][1],CASES[0][2]))
def snap():return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()}
before=snap();env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch');env.pop('CARGO_TARGET_DIR',None)
if a.no_tmp:env.pop('TMPDIR',None)
r=subprocess.run([a.binary,'do',str(root),a.request],env=env,text=True,capture_output=True,timeout=110)
(out/'output.log').write_text(r.stdout+r.stderr)
result=dict(accepted='Applied test-verified proposal' in r.stdout,unchanged=before==snap(),before=before,after=snap(),output=r.stdout)
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
