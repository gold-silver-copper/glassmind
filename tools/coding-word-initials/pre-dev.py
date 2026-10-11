"""Require every prospectively frozen local gate before full public DEV."""
import hashlib,json,re,sys
from pathlib import Path
r=Path(sys.argv[1]).resolve();w=Path.cwd()
read=lambda name:json.loads((r/name).read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
candidate=read('candidate.json')
assert candidate['sha256']==sha(r/'candidate')==sha(Path('/home/gsc/code/glassmind/target/release/glassmind'))
for path,digest in candidate['sources'].items():assert sha(w/path)==digest
native=read('native/results.json');verified=read('native-verification/results.json')
assert len(native)==23 and sum(x['accepted'] for x in native)==17
assert {x['id'] for x in verified}=={x['id'] for x in native if x['accepted']}
assert all(x['ok'] for x in verified)
assert read('ambiguity-check.json')['passing_candidates']>1 and read('ambiguity-check.json')['unchanged']
unchanged={}
for name in ['native','old-native']:
 old={x['id']:x for x in json.loads((r.parent/'word-join'/name/'results.json').read_text())}
 rows=read(name+'/results.json');checked=0
 for row in rows:
  if name=='native' and not row['id'].startswith('old-'):continue
  key=row['id'].removeprefix('old-') if name=='native' else row['id']
  assert all(row[k]==old[key][k] for k in ['before','after','accepted'])
  checked+=1
 assert checked==(10 if name=='native' else 20)
 unchanged[name]=checked
for name in ['independent','unset-target']:
 rows=read(name+'/report.json')['cases'];assert len(rows)==18 and sum(x['accepted'] for x in rows)==6
 assert all(not x['changed'] for x in rows if x['kind']!='implement')
extra=read('extra/report.json');assert len(extra)==10 and sum(x['accepted'] for x in extra)==3
train=read('train-comparison.json');assert train['total']==60 and len(train['retained'])==5 and len(train['gained'])>=1
assert read('train-sample/report.json')['total']==9
build=(r/'build.log').read_text();tests=sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed',build));assert tests==59
workspace=(r/'workspace.log').read_text();assert "Fixture smoke: {'turns': 4, 'right': 4, 'refused': 0, 'wrong': 0" in workspace
names=['build.log','workspace.log','native/report.json','native-verification/results.json','ambiguity-check.json','old-native/report.json','independent/report.json','unset-target/report.json','extra/report.json','train-sample/report.json','train-full/report.json','train-full-verification/results.json','train-comparison.json']
report=dict(candidate=candidate,world_tests=tests,native_cases=23,existing_cases=76,unchanged_old_source_outcomes=unchanged,train_total=60,train_accepted=train['accepted'],all_pre_dev_gates_passed=True,evidence={name:dict(bytes=(r/name).stat().st_size,sha256=sha(r/name)) for name in names})
with (r/'pre-dev.json').open('x') as f:f.write(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(all_pre_dev_gates_passed=True,world_tests=tests,new_implementations=12,train_accepted=train['accepted'])))
