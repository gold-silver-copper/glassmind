from pathlib import Path
import argparse,hashlib,json,os,signal,subprocess,collections,re
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
base=Path(__file__).parent;raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];cases=json.loads(raw)
a.out.mkdir(parents=True,exist_ok=False);a.out=a.out.resolve();binary=a.binary.resolve();rows=[]
def normalize(expression):
 expression=''.join(expression.split())
 def outer(s):
  if not (s.startswith('(') and s.endswith(')')):return False
  depth=0
  for i,c in enumerate(s):
   depth+=int(c=='(')-int(c==')')
   if depth==0:return i==len(s)-1
  return False
 while outer(expression):expression=expression[1:-1]
 while True:
  revised=re.sub(r'(?<![\w])\(([\w.]+)\)',r'\1',expression)
  if revised==expression:break
  expression=revised
 return expression

def exact_source(case,after):
 if case['expected']!='repair':return after==case['source']
 original=case['source'];expression=case['expression'];assert original.count(expression)==1
 prefix,suffix=original.split(expression)
 if not after.startswith(prefix) or not after.endswith(suffix):return False
 middle=after[len(prefix):len(after)-len(suffix)]
 lhs=case['left'];rhs=case['right'];ty=case['primitive']
 allowed=[f'::core::primitive::{ty}::div_ceil({lhs},{rhs})',f'::std::primitive::{ty}::div_ceil({lhs},{rhs})',f'({lhs}).div_ceil({rhs})']
 return normalize(middle) in {normalize(x) for x in allowed}

for c in cases:
 case=a.out/c['name'];root=case/'workspace';(root/'src').mkdir(parents=True)
 manifest='[package]\nname="parent_'+c['name']+'"\nversion="0.1.0"\nedition="2024"\n'
 (root/'Cargo.toml').write_text(manifest);(root/'src/lib.rs').write_text(c['source'])
 env=dict(os.environ,TMPDIR=str(case/'tmp'),GLASSMIND_SCRATCH=str(case/'scratch'),GLASSMIND_CARGO_TARGET=str(case/'check-target'),GLASSMIND_CHECK_CACHE=str(case/'check-cache'),CARGO_TARGET_DIR=str(case/'native-target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4');Path(env['TMPDIR']).mkdir()
 def run(cmd,label):
  with (case/(label+'.stdout')).open('w') as out,(case/(label+'.stderr')).open('w') as err:
   proc=subprocess.Popen(cmd,cwd=root,env=env,stdout=out,stderr=err,start_new_session=True)
   try:rc=proc.wait(timeout=160)
   except subprocess.TimeoutExpired:
    os.killpg(proc.pid,signal.SIGTERM)
    try:proc.wait(timeout=10)
    except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
    rc=124
  return rc
 cargo=['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120','cargo','test','--offline','-j1','--','--test-threads','4']
 before=run(cargo,'original-test')
 action=run([str(binary),'do',str(root),'--explain','Fix the failing tests.'],'action')
 after=(root/'src/lib.rs').read_text();changed=after!=c['source'];native=run(cargo,'after-test') if changed else None
 witnesses=[]
 for f in (case/'scratch').glob('ambiguity-*/witness.json'):
  r=json.loads(f.read_text());witnesses.append({'path':str(f),'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'native_witness':r.get('native_witness'),'witness':r.get('witness')})
 row=dict(name=c['name'],expected=c['expected'],original_test_exit=before,action_exit=action,changed=changed,exact_source=exact_source(c,after),manifest_unchanged=(root/'Cargo.toml').read_text()==manifest,native_exit=native,witnesses=witnesses,after_source=after)
 rows.append(row);(a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),cases=rows),indent=2)+'\n')
 print(c['name'],'original',before,'changed',changed,'exact',row['exact_source'],'native',native,'witness',len(witnesses),flush=True)
summary=dict(total=len(rows),original_test_failures=sum(r['original_test_exit']==101 for r in rows),repairs=sum(r['expected']=='repair' and r['exact_source'] and r['native_exit']==0 for r in rows),repair_total=sum(r['expected']=='repair' for r in rows),negative_edits=[r['name'] for r in rows if r['expected']=='unchanged' and r['changed']],ambiguity_withdrawn=all(not r['changed'] and r['witnesses'] for r in rows if r['expected']=='ambiguous'),timeouts=[r['name'] for r in rows if r['action_exit']==124],manifest_unchanged=all(r['manifest_unchanged'] for r in rows))
(a.out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary),flush=True)
