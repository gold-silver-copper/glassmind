"""Replay frozen authored cases through actual main binary and native tests."""
import argparse,hashlib,json,os,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--out',required=True);p.add_argument('--contracts',default=str(Path(__file__).with_name('contracts.json')));a=p.parse_args()
out=Path(a.out).resolve();out.mkdir(parents=True,exist_ok=True)
env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
env.update(GLASSMIND_SCRATCH=str(out/'scratch'),GLASSMIND_CARGO_TARGET=str(Path(env['CARGO_TARGET_DIR'])/'authored'),GLASSMIND_CHECK_CACHE=str(out/'checks'))
results=[]
for c in json.loads(Path(a.contracts).read_text()):
 root=out/'sources'/c['name'];(root/'src').mkdir(parents=True,exist_ok=True);(root/'tests').mkdir(exist_ok=True)
 (root/'Cargo.toml').write_text('[package]\nname="probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n')
 (root/'src/lib.rs').write_text(c['code']);(root/'tests/visible.rs').write_text('#[test] fn contract(){'+c['test']+'}\n')
 before={str(q.relative_to(root)):hashlib.sha256(q.read_bytes()).hexdigest() for q in root.rglob('*') if q.is_file()}
 inp=out/'input.jsonl';inp.write_text(json.dumps(dict(id=c['name'],sources=[c['name']],turns=[dict(id=c['name']+'#0',text='Fix the failing tests.')]))+'\n')
 with (out/(c['name']+'.stderr')).open('w') as log:
  proc=subprocess.run([a.binary,'eval','--sources',str(out/'sources'),'--inputs',str(inp),'--jobs','1'],env=env,text=True,capture_output=True,timeout=180)
  log.write(proc.stderr)
 (out/(c['name']+'.outcome.json')).write_text(proc.stdout)
 result=json.loads(proc.stdout);changes=result['response'].get('changes',[])
 unchanged=all(hashlib.sha256((root/q).read_bytes()).hexdigest()==h for q,h in before.items())
 expected=c['code'].replace(c['old'],c['new'],1) if c['old'] else c['code']
 actual=c['code']
 for change in sorted(changes,key=lambda x:(x['line'],x['column']),reverse=True):
  assert change['file']=='src/lib.rs'
  offset=sum(map(len,actual.splitlines(keepends=True)[:change['line']-1]))+change['column'];assert actual[offset:offset+len(change['old'])]==change['old']
  actual=actual[:offset]+change['new']+actual[offset+len(change['old']):]
 verified=None
 if changes:
  (root/'src/lib.rs').write_text(actual)
  with (out/(c['name']+'.verify.log')).open('w') as log:
   verified=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--test-threads','4'],cwd=root,env=dict(env,CARGO_TARGET_DIR=str(Path(env['CARGO_TARGET_DIR'])/'authored-verify'/c['name'])),stdout=log,stderr=subprocess.STDOUT).returncode==0
 results.append(dict(name=c['name'],positive=c['old'] is not None,changes=changes,original_unchanged=unchanged,exact=actual==expected,verified=verified,source_sha256=before))
 (out/'report.json').write_text(json.dumps(results,indent=2)+'\n');print(c['name'],bool(changes),actual==expected,unchanged,flush=True)
