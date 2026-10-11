"""Replay frozen authored fixtures through main CLI; retain every input/output."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=pathlib.Path,required=True);p.add_argument('--out',type=pathlib.Path,required=True);p.add_argument('--baseline',action='store_true');p.add_argument('--fixtures',type=pathlib.Path,default=pathlib.Path(__file__).parent/'fixtures.json');a=p.parse_args()
a.out.mkdir(parents=True,exist_ok=True)
env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
for key,sub in [('TMPDIR','tmp'),('GLASSMIND_SCRATCH','scratch'),('GLASSMIND_CHECK_CACHE','checks')]:
 env[key]=str((a.out/sub).resolve());pathlib.Path(env[key]).mkdir(exist_ok=True)
env['GLASSMIND_CARGO_TARGET']=str(pathlib.Path(env['CARGO_TARGET_DIR'])/'authored')
env['CARGO_NET_OFFLINE']='true'
def snap(root): return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()}
results=[]
for c in json.loads(a.fixtures.read_text()):
 root=a.out/c['name'];root.mkdir();
 for f,t in c['files'].items():
  path=root/f;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(t)
 before=snap(root)
 r=subprocess.run([str(a.binary.resolve()),'do',str(root.resolve()),'Fix the failing tests.'],env=env,text=True,capture_output=True,timeout=240)
 (a.out/(c['name']+'.stdout')).write_text(r.stdout);(a.out/(c['name']+'.stderr')).write_text(r.stderr)
 after=snap(root); expected=dict(c['files'])
 if c['old']: expected[c['file']]=expected[c['file']].replace(c['old'],c['new'],1)
 exact=snap(root)=={f:hashlib.sha256(t.encode()).hexdigest() for f,t in expected.items()}
 changed=before!=after
 wanted=c['kind'] in ('positive','legacy') and not (a.baseline and c['kind']=='positive')
 ok=exact if wanted else not changed
 if c['kind']=='ambiguity' and a.baseline: ok=changed
 verify=None
 if changed:
  verify_env=dict(env,CARGO_TARGET_DIR=str(pathlib.Path(env['CARGO_TARGET_DIR'])/'authored-verify'/c['name']))
  with (a.out/(c['name']+'.verify.log')).open('w') as log:
   v=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--test-threads','4'],cwd=root,env=verify_env,stdout=log,stderr=subprocess.STDOUT)
  verify=v.returncode;ok=ok and verify==0
 results.append(dict(name=c['name'],kind=c['kind'],ok=ok,exact=exact,changed=changed,returncode=r.returncode,verify=verify,before=before,after=after))
 (a.out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
 print(c['name'],ok,'changed',changed,flush=True)

raise SystemExit(0 if all(r["ok"] for r in results) else 1)
