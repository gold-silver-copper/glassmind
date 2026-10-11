"""Bounded main-binary ambiguity replay of public authored contracts only.

Public native tests are not proof of intended semantics. No parent cases/logs,
held-out assets, labels, or evaluation definitions are accessed.
"""
import argparse, hashlib, json, os, shutil, subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
r=a.out.resolve();r.mkdir(parents=True,exist_ok=True)
env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
target=Path(env['CARGO_TARGET_DIR']);env.update(TMPDIR=str(r/'tmp'),GLASSMIND_SCRATCH=str(r/'scratch'),GLASSMIND_CARGO_TARGET=str(target/'ambiguity-replay'),GLASSMIND_CHECK_CACHE=str(r/'checks'))
Path(env['TMPDIR']).mkdir(exist_ok=True)
wrapper=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo']
def cargo(root,name,args):
 result=subprocess.run(wrapper+args, cwd=root,env=dict(env,CARGO_TARGET_DIR=str(target/'ambiguity-native'/root.name)),text=True,capture_output=True)
 (root/f'{name}.stdout').write_text(result.stdout);(root/f'{name}.stderr').write_text(result.stderr);return result
own=json.loads(Path('tools/coding-dedup-order/cases.json').read_text())
old=json.loads(Path('tools/coding-argument-swap/contracts.json').read_text())
pipeline=next(c for c in own if c['name']=='two_passing')
swap=next(c for c in old if c['name']=='two_passing')
records=[]
for name,c in [('pipeline',pipeline),('old-argument-swap',swap)]:
 root=r/name;(root/'src').mkdir(parents=True,exist_ok=True);(root/'tests').mkdir(exist_ok=True)
 code=c['code'];(root/'src/lib.rs').write_text(code)
 (root/'Cargo.toml').write_text('[package]\nname="probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n')
 tests='use probe::*;\n'+c['test'] if name=='pipeline' else '#[test] fn check(){'+c['test']+'}'
 (root/'tests/visible.rs').write_text(tests)
 before=cargo(root,'before',['test','-j1','--','--test-threads','4'])
 assert before.returncode==101 and 'test result: FAILED' in before.stdout and 'could not compile' not in before.stderr
 variants=[]
 if name=='pipeline':
  oldtext='xs.dedup(); /* preserved gap */ xs.sort();';newtext='xs.sort(); /* preserved gap */ xs.dedup();'
  starts=[i for i in range(len(code)) if code.startswith(oldtext,i)]
  variants=[code[:i]+newtext+code[i+len(oldtext):] for i in starts]
  output='fn main(){println!("{:?} {:?}",probe::left(vec![2,1,2]),probe::right(vec![2,1,2]));}'
  expected=['[1, 2] [1, 2, 2]\n','[1, 2, 2] [1, 2]\n']
 else:
  variants=[code.replace('pair(b,a)','pair(a,b)',1),code.replace('pair(d,c)','pair(c,d)',1)]
  output='fn main(){println!("{:?}",probe::run(1,2,3,4));}'
  expected=['((1, 2), (4, 3))\n','((2, 1), (3, 4))\n']
 outputs=[]
 for i,variant in enumerate(variants):
  v=r/f'{name}-variant-{i}';shutil.copytree(root,v,dirs_exist_ok=True);(v/'src/lib.rs').write_text(variant)
  after=cargo(v,'native',['test','-j1','--','--test-threads','4']);assert after.returncode==0
  (v/'examples').mkdir(exist_ok=True);(v/'examples/output.rs').write_text(output)
  result=cargo(v,'output',['run','-j1','--quiet','--example','output']);assert result.returncode==0;outputs.append(result.stdout)
 assert outputs==expected
 before_files={str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest() for f in [root/'src/lib.rs',root/'tests/visible.rs',root/'Cargo.toml']}
 result=subprocess.run([str(a.binary.resolve()),'do',str(root),'--explain','Fix the failing tests.'],env=env,text=True,capture_output=True,timeout=180)
 (root/'action.stdout').write_text(result.stdout);(root/'action.stderr').write_text(result.stderr)
 assert result.returncode==0,(result.returncode,result.stderr)
 for f,digest in before_files.items():assert hashlib.sha256((root/f).read_bytes()).hexdigest()==digest,(name,f)
 choices=[]
 for path in (r/'scratch').glob('*/choice.json'):
  record=json.loads(path.read_text())
  if name=='pipeline' and 'dedup-order-' in path.parent.name:
   assert record['accepted'] is False and record['incomplete'] is False and len(record['passing'])==2
   assert len(record['native_checks'])==2 and all(x['result']=={'Ok':[]} for x in record['native_checks'])
   choices.append(str(path.relative_to(r)))
  if name=='old-argument-swap' and 'unresolved-' in path.parent.name:
   assert record['second_tests_passed'] is True
   choices.append(str(path.relative_to(r)))
 assert choices,(name,result.stdout,result.stderr)
 records.append(dict(name=name,compiled_original_executed_failure=True,both_alternatives_native_passed=True,native_outputs=outputs,end_to_end_refused=True,choices=choices,original_sha256=hashlib.sha256(code.encode()).hexdigest()))
 (r/'report.json').write_text(json.dumps(records,indent=2)+'\n');print(name,'both native passed, outputs differ, main refused',flush=True)
