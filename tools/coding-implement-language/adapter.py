#!/usr/bin/env python3
"""TRAIN input-only/public tests replay, or independently frozen language contracts.
Tests are copied/executed opaquely; no assertion inspection or expected patches.
"""
import argparse, hashlib, json, os, pathlib, shutil, subprocess, time
from original.fixtures import MANIFEST, source
BASE=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--out',required=True);p.add_argument('--mode',choices=['prepare','sample','train','contracts'],required=True);p.add_argument('--no-env',action='store_true');a=p.parse_args()
out=pathlib.Path(a.out).resolve();out.mkdir(parents=True,exist_ok=False)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def snap(root):return {str(p.relative_to(root)):dict(sha256=digest(p),bytes=p.stat().st_size) for p in sorted(root.rglob('*')) if p.is_file()}
def train():
 r=pathlib.Path('/home/gsc/.cache/glassmind/data/train-v3')
 # Projection to metadata only; expected label fields are never consulted.
 allowed={x['conversation'] for l in (r/'labels.jsonl').open() if (x:=json.loads(l)).get('split')=='train' and x.get('kind')=='rb_implement'}
 result=[]
 for l in (r/'inputs.jsonl').open():
  x=json.loads(l)
  if x['id'] in allowed:
   for t in x['turns']:result.append(dict(id=t['id'],text=t['text'],source=x['sources'][0]))
 return result
if a.mode=='contracts':
 c=json.loads((BASE/'contracts.json').read_text());rows=[dict(id=f'accept-{i}',text=t,target=n,expect=True) for i,(t,n) in enumerate(c['accept'])]+[dict(id=f'refuse-{i}',text=t,target='x',expect=False) for i,t in enumerate(c['refuse'])]
else:
 rows=train()
 if a.mode=='sample':rows=[r for r in rows if r['id']=='train/rb-implement-sum-even/terse#0']
results=[]
for row in rows:
 root=out/row['id'].replace('/','_').replace('#','_');(root/'src').mkdir(parents=True)
 if a.mode=='contracts':
  (root/'Cargo.toml').write_text(MANIFEST)
  n=row['target'];(root/'src/lib.rs').write_text(source(f'pub fn {n}(xs: &[i32]) -> i32 {{ todo!() }}',f'assert_eq!({n}(&[-7,2,4,9]),8); assert_eq!({n}(&[-3,-2]),-5);'))
 else:
  src=pathlib.Path('/home/gsc/.cache/glassmind/sources')/row['source']
  shutil.copyfile(src/'Cargo.toml',root/'Cargo.toml')
  # Allowlist production workspace only; never hidden/config/task/generator assets.
  for folder in ['src','tests']:
   if (src/folder).is_dir():
    for f in (src/folder).rglob('*.rs'):
     rel=f.relative_to(src)
     if any(x.startswith('.') for x in rel.parts):continue
     dest=root/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dest)
 before=snap(root)
 if a.mode=='prepare':
  results.append(dict(**row,files=before));continue
 env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
 if a.no_env:
  env.pop('CARGO_TARGET_DIR',None);env.pop('TMPDIR',None)
 start=time.monotonic()
 r=subprocess.run([a.binary,'do',str(root),row['text']],env=env,capture_output=True,text=True,timeout=110)
 (out/(root.name+'.log')).write_text(r.stdout+r.stderr)
 accepted='Applied test-verified proposal' in r.stdout
 after=snap(root);unchanged=before==after;preserved=True
 if accepted:
  original=(pathlib.Path('/home/gsc/.cache/glassmind/sources')/row['source']/'src/lib.rs').read_bytes() if a.mode!='contracts' else source(f"pub fn {row['target']}(xs: &[i32]) -> i32 {{ todo!() }}",f"assert_eq!({row['target']}(&[-7,2,4,9]),8); assert_eq!({row['target']}(&[-3,-2]),-5);").encode()
  left,right=original.split(b'todo!()',1);actual=(root/'src/lib.rs').read_bytes()
  preserved=actual.startswith(left) and actual.endswith(right) and {k:v for k,v in before.items() if k!='src/lib.rs'}=={k:v for k,v in after.items() if k!='src/lib.rs'}
  verify_env=dict(env);verify_env['TMPDIR']=os.environ['TMPDIR'];verify_env['CARGO_TARGET_DIR']=os.environ['CARGO_TARGET_DIR']
  cmd=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4']
  v=subprocess.run(cmd,cwd=root,env=verify_env,capture_output=True,text=True,timeout=620)
  (out/(root.name+'-verify.log')).write_text(v.stdout+v.stderr)
  preserved=preserved and v.returncode==0
 ok=preserved if accepted else unchanged
 if 'expect' in row:ok=ok and accepted==row['expect']
 result=dict(**row,accepted=accepted,ok=ok,unchanged=unchanged,before=before,after=after,seconds=round(time.monotonic()-start,3),exit=r.returncode)
 results.append(result);print(json.dumps({k:v for k,v in result.items() if k not in ['before','after']}),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2,ensure_ascii=False)+'\n')
raise SystemExit(0 if all(r.get('ok',True) for r in results) else 1)
