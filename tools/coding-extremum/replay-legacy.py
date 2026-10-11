"""Replay original public authored contracts without rewriting expectations."""
import argparse,json,os,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--supplement-only',action='store_true');p.add_argument('--binary',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();out=a.out.resolve();out.mkdir(parents=True,exist_ok=True)
env={k:v for k,v in os.environ.items() if 'API_KEY' not in k};target=Path(env['CARGO_TARGET_DIR']);env.update(GLASSMIND_SCRATCH=str(out/'scratch'),GLASSMIND_CARGO_TARGET=str(target/'legacy-checks'),GLASSMIND_CHECK_CACHE=str(out/'checks'))
worker=Path('tools/coding-argument-swap/contracts.json');parent=Path('tools/coding-argument-swap/parent/cases.json')
rows=[]
for group,path in ([('worker25',Path('tools/coding-argument-swap/supplement.json'))] if a.supplement_only else [('worker25',worker),('worker25',Path('tools/coding-argument-swap/supplement.json')),('parent13',parent)]):
 for c in json.loads(path.read_text()):
  name=group+'-'+c['name'];root=out/name;(root/'src').mkdir(parents=True,exist_ok=True)
  code=c['code'] if group=='worker25' else c['source'];(root/'src/lib.rs').write_text(code)
  (root/'Cargo.toml').write_text('[package]\nname="probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n')
  if group=='worker25':
   (root/'tests').mkdir(exist_ok=True);(root/'tests/visible.rs').write_text('#[test] fn contract(){'+c['test']+'}\n')
  def cargo(label):
   with (root/(label+'.log')).open('w') as log:
    return subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--test-threads','4'],cwd=root,env=dict(env,CARGO_TARGET_DIR=str(target/'legacy-native'/name)),stdout=log,stderr=subprocess.STDOUT).returncode
  before=cargo('before');log=(root/'before.log').read_text();assert before==101 and 'test result: FAILED' in log and 'could not compile' not in log,(name,log)
  result=subprocess.run([a.binary,'do',str(root),'--explain','Fix the failing tests.'],env=env,text=True,capture_output=True,timeout=180)
  (root/'action.stdout').write_text(result.stdout);(root/'action.stderr').write_text(result.stderr)
  after=(root/'src/lib.rs').read_text();repair=bool(c.get('old')) if group=='worker25' else c['kind']=='repair'
  if repair:
   expected=code.replace(c['old'],c['new'],1)
   if group=='parent13':
    prefix,suffix=code.split(c['old']);exact=after.startswith(prefix) and after.endswith(suffix) and ''.join(after[len(prefix):len(after)-len(suffix)].split())==''.join(c['new'].split())
   else:exact=after==expected
  else:exact=after==code
  native=cargo('after') if after!=code else None
  rows.append(dict(name=name,exact=exact,changed=after!=code,native=native,initial_executed_failure=True,contracts_sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
  (out/'report.json').write_text(json.dumps(rows,indent=2)+'\n');print(name,exact,native,flush=True)
assert all(r['exact'] and (not r['changed'] or r['native']==0) for r in rows)
