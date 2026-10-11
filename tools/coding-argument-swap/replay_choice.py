"""Independently replay each unresolved choice against unchanged authored tests."""
import argparse,hashlib,json,os,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);p.add_argument('--label',default='candidate');a=p.parse_args()
record=json.loads(next((a.run/('authored-'+a.label)/'scratch').glob('unresolved-*/choice.json')).read_text())
contract=next(c for c in json.loads(Path(__file__).with_name('contracts.json').read_text()) if c['name']=='two_passing')
original=a.run/('authored-'+a.label)/'sources/two_passing'
assert (original/'src/lib.rs').read_text()==contract['code']
results=[]
for side in ['first','second']:
 root=a.run/('choice-replay-'+a.label)/side
 shutil.copytree(original,root,dirs_exist_ok=True)
 change=record[side+'_patch'];text=contract['code']
 offset=sum(map(len,text.splitlines(keepends=True)[:change['line']-1]))+change['column']
 assert text[offset:offset+len(change['old'])]==change['old']
 text=text[:offset]+change['new']+text[offset+len(change['old']):]
 (root/'src/lib.rs').write_text(text)
 env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
 env['CARGO_TARGET_DIR']=str(Path(env['CARGO_TARGET_DIR'])/'choice-replay'/side)
 with (root/'test.log').open('w') as log:
  result=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--test-threads','4'],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append(dict(side=side,exit_code=result.returncode,source_sha256=hashlib.sha256(text.encode()).hexdigest(),test_sha256=hashlib.sha256((root/'tests/visible.rs').read_bytes()).hexdigest(),patch=change))
 assert result.returncode==0
(a.run/('choice-replay-'+a.label)/'report.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
