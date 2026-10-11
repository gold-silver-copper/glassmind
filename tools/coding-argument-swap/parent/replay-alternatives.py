from pathlib import Path
import json,subprocess,os,hashlib
base=Path(__file__).parent;case=next(c for c in json.loads((base/'cases.json').read_text()) if c['kind']=='unresolved');out=base/'alternatives';out.mkdir();env={k:v for k,v in os.environ.items() if 'API_KEY' not in k};env.update(TMPDIR=str(base/'tmp'),CARGO_TARGET_DIR=str(out/'target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4');records=[]
for i,((old,new),expected) in enumerate(zip(case['alternatives'],case['expected_outputs'],strict=True)):
 root=out/f'alternative-{i}';(root/'src/bin').mkdir(parents=True);(root/'Cargo.toml').write_text('[package]\nname="argument_swap_native"\nversion="0.1.0"\nedition="2024"\n');assert case['source'].count(old)==1;edited=case['source'].replace(old,new);(root/'src/lib.rs').write_text(edited);(root/'src/bin/probe.rs').write_text('fn main(){println!("{}",argument_swap_native::value(7,2));}\n')
 for label,cmd in [('test',['cargo','test','--offline','-j1','--','--test-threads','4']),('probe',['cargo','run','--offline','--quiet','-j1','--bin','probe'])]:
  wrapped=['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120',*cmd];r=subprocess.run(wrapped,cwd=root,env=env,text=True,capture_output=True,timeout=140);(root/f'{label}.stdout').write_text(r.stdout);(root/f'{label}.stderr').write_text(r.stderr);assert r.returncode==0,r.stderr
  if label=='probe':observed=int(r.stdout.strip());assert observed==expected
 records.append({'alternative':i,'old':old,'new':new,'public_tests_passed':True,'input':case['distinguishing_input'],'observed_output':observed,'source_sha256':hashlib.sha256(edited.encode()).hexdigest()})
assert records[0]['observed_output']!=records[1]['observed_output'];(out/'report.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(records,indent=2))
