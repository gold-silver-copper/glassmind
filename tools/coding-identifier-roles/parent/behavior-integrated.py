"""Execute preserved behavior tests for all safe integrated spelling edits."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/identifierroles-review');report=json.loads((r/'spelling-integrated/report.json').read_text());assert report['safe']==8;out=[]
for i,row in enumerate(report['cases']):
 if not row['mutated']:continue
 source=r/'spelling-integrated'/str(i);dest=r/'spelling-behavior'/str(i);shutil.copytree(source,dest)
 env=dict(os.environ);env['CARGO_TARGET_DIR']=str(r/'spelling-behavior-target'/str(i));env['CARGO_BUILD_JOBS']='1';env['RUST_TEST_THREADS']='4'
 v=subprocess.run(['cargo','test','--offline','--manifest-path',str(dest/'Cargo.toml'),'-j1','--','--test-threads','4'],env=env,text=True,capture_output=True,timeout=80);(r/'spelling-behavior'/f'{i}.log').write_text(v.stdout+v.stderr)
 assert v.returncode==0 and 'test same ... ok' in v.stdout and '1 passed; 0 failed' in v.stdout
 assert (source/'src/lib.rs').read_bytes()==(dest/'src/lib.rs').read_bytes()
 out.append(dict(index=i,request=row['request'],exit=v.returncode,native_test_executed=True,source_sha256=hashlib.sha256((dest/'src/lib.rs').read_bytes()).hexdigest()))
assert len(out)==7
(r/'spelling-behavior.json').write_text(json.dumps(dict(native_tests_passed=len(out),cases=out),indent=2)+'\n');print('7 integrated spelling edits pass the unchanged native behavior test')
