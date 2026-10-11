"""Frozen exact-byte and native behavior contracts; baseline misses are reported."""
import argparse, hashlib, json, os, subprocess
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--sample',action='store_true');p.add_argument('--controls',action='store_true');a=p.parse_args()
assert os.environ['TMPDIR'] and not os.environ['TMPDIR'].startswith('/tmp')
assert os.environ['CARGO_TARGET_DIR'] and os.environ['GLASSMIND_SCRATCH']
scope=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
cases=json.loads(Path(__file__).with_name('contracts.json').read_text())
if a.controls: cases=[c for c in cases if 'control' in c['id'] or 'backtick' in c['id']]
if a.sample: cases=[c for c in cases if c['id'] in ['query-caller-plain','query-caller-backtick','rename-implement','rename-control-implement','field-query-called','field-control-called','negation','collision']]
a.output.mkdir(parents=True,exist_ok=False)
rows=[]
def run(args,root):
 r=subprocess.run(scope+args,cwd=root,text=True,capture_output=True,timeout=620)
 return dict(code=r.returncode,stdout=r.stdout,stderr=r.stderr)
for case in cases:
 root=a.output/case['id'];root.mkdir()
 for name,content in case['files'].items():
  f=root/name; f.parent.mkdir(parents=True,exist_ok=True);f.write_text(content)
 before=run(['cargo','test','-j1','--','--test-threads','4'],root) if case['positive'] else None
 result=run([str(a.binary.resolve()),'do',str(root.resolve()),case['request']],root)
 actual={name:(root/name).read_text() for name in case['files']}
 changed=actual!=case['files']
 after=run(['cargo','test','-j1','--','--test-threads','4'],root) if changed else before
 exact=actual==case['expected']
 behavior=not case['positive'] or all(r and r['code']==0 and '1 passed' in r['stdout'] for r in [before,after])
 passed=exact and behavior and (case['reply'] is None or result['stdout']==case['reply']) and (not case['mutation'] or changed)
 row=dict(id=case['id'],request=case['request'],pass_=passed,changed=changed,exact=exact,behavior=behavior,result=result,before=before,after=after,actual=actual)
 rows.append(row); print(case['id'],passed,repr(result['stdout']),flush=True)
(a.output/'results.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(dict(total=len(rows),passed=sum(r['pass_'] for r in rows),changed=sum(r['changed'] for r in rows))))
