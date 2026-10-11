#!/usr/bin/env python3
"""Independent environment failure checks on authored fixtures; no benchmark data."""
import argparse,hashlib,json,os,pathlib,subprocess
from original.fixtures import MANIFEST,CASES,source
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--out',required=True);a=p.parse_args()
out=pathlib.Path(a.out).resolve();out.mkdir(parents=True,exist_ok=False)
results=[]
for mode in ['no-target','no-target-or-tmp','tmp-in-source','target-in-source','relative-target','tmp-symlink']:
 root=out/mode;(root/'src').mkdir(parents=True)
 code=source(CASES[0][1],CASES[0][2]);(root/'Cargo.toml').write_text(MANIFEST);(root/'src/lib.rs').write_text(code)
 def snap():return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()}
 before=snap();env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
 if mode in ['no-target','no-target-or-tmp']:env.pop('CARGO_TARGET_DIR',None)
 if mode=='no-target-or-tmp':env.pop('TMPDIR',None)
 if mode=='tmp-in-source':env['TMPDIR']=str(root/'new-tmp')
 if mode=='target-in-source':env['CARGO_TARGET_DIR']=str(root/'new-target')
 if mode=='relative-target':env['CARGO_TARGET_DIR']='relative-target'
 if mode=='tmp-symlink':
  link=out/'tmp-link';link.symlink_to('/tmp',target_is_directory=True);env['TMPDIR']=str(link/'must-not-create-glassmind-impllanguage')
 r=subprocess.run([a.binary,'do',str(root),'implement `aggregate`'],env=env,text=True,capture_output=True,timeout=110)
 (out/(mode+'.log')).write_text(r.stdout+r.stderr)
 accepted='Applied test-verified proposal' in r.stdout;positive=mode.startswith('no-target')
 ok=accepted if positive else (not accepted and before==snap() and not (root/'new-tmp').exists() and not (root/'new-target').exists())
 results.append(dict(case=mode,accepted=accepted,ok=ok,before=before,after=snap()));print(json.dumps(results[-1]),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
raise SystemExit(0 if all(x['ok'] for x in results) else 1)
