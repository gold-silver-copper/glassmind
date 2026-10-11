from pathlib import Path
import hashlib,json,os,shutil,subprocess
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/identifier-spelling');report=json.loads((r/'candidate/report.json').read_text());base=json.loads((r.parent/'identifierroles-review/baseline/report.json').read_text());rows=[]
assert report['safe']==8 and report['mutations']==6
for i,(old,row) in enumerate(zip(base['cases'],report['cases'],strict=True)):
 assert row['safe']
 if old['safe']:assert old['before']==row['before'] and old['after']==row['after']
 if not row['mutated']:continue
 source=r/'candidate'/str(i);copy=r/'behavior'/str(i);shutil.copytree(source,copy)
 env=dict(os.environ);env['CARGO_TARGET_DIR']=str(r/'behavior-target'/str(i));env['CARGO_BUILD_JOBS']='1';env['RUST_TEST_THREADS']='4'
 v=subprocess.run(['cargo','test','--offline','--manifest-path',str(copy/'Cargo.toml'),'-j1','--','--test-threads','4'],env=env,text=True,capture_output=True,timeout=80)
 (r/'behavior'/f'{i}.log').write_text(v.stdout+v.stderr)
 executed='test same ... ok' in v.stdout and '1 passed; 0 failed' in v.stdout
 rows.append(dict(index=i,request=row['request'],exit=v.returncode,test_executed=executed,source_sha256=hashlib.sha256((source/'src/lib.rs').read_bytes()).hexdigest(),behavior_source_sha256=hashlib.sha256((copy/'src/lib.rs').read_bytes()).hexdigest()))
 assert v.returncode==0 and executed,rows[-1]
 assert (source/'src/lib.rs').read_bytes()==(copy/'src/lib.rs').read_bytes()
out=dict(exact_checks=8,safe=8,previously_correct_retained=5,wrong_targets_corrected=3,accepted_edits=6,native_tests_executed=len(rows),cases=rows)
(r/'behavior.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
