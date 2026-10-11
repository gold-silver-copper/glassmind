"""Enforce the frozen native/TRAIN gates before full public DEV."""
import hashlib,json,re,sys
from pathlib import Path
r=Path(sys.argv[1]).resolve();previous=r.parent/'word-initials'
read=lambda name:json.loads((r/name).read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
candidate=read('candidate.json')
assert candidate['sha256']==sha(r/'candidate')
for path,digest in candidate['sources'].items():assert sha(Path(path))==digest,path
assert read('baseline-native/report.json')['accepted']==0
native=read('native/results.json')
assert len(native)==22 and sum(x['accepted'] for x in native)==16
weak=next(x for x in native if x['id']=='weak-ambiguity')
assert not weak['accepted'] and '16 passing candidates' in weak['stdout'] and weak['before']==weak['after']
for name in ['native','train-full']:
 rows=read(name+'/results.json');verified=read(name+'-verification/results.json')
 assert {x['id'] for x in verified}=={x['id'] for x in rows if x['accepted']}
 assert all(x['ok'] for x in verified)
unchanged={}
for new,old in [('old-initials','native'),('old-native','old-native')]:
 rows=read(new+'/results.json');before={x['id']:x for x in json.loads((previous/old/'results.json').read_text())}
 assert len(rows)==len(before)==(23 if new=='old-initials' else 20)
 for row in rows:assert all(row[k]==before[row['id']][k] for k in ['before','after','accepted']),row['id']
 unchanged[new]=len(rows)
for name in ['independent','unset-target']:
 rows=read(name+'/report.json')['cases'];old=json.loads((previous/name/'report.json').read_text())['cases']
 assert len(rows)==len(old)==18 and sum(x['accepted'] for x in rows)==6
 for row,prior in zip(rows,old):
  assert all(row[k]==prior[k] for k in ['name','kind','accepted','changed','source']),row['name']
 unchanged[name]=len(rows)
rows=read('extra/report.json');old=json.loads((previous/'extra/report.json').read_text())
assert len(rows)==len(old)==10 and sum(x['accepted'] for x in rows)==3
for row,prior in zip(rows,old):assert all(row[k]==prior[k] for k in ['request','accepted','before','after'])
unchanged['extra']=len(rows)
assert sum(unchanged.values())==89
old={x['id']:x for x in json.loads((previous/'train-full/results.json').read_text())}
rows=read('train-full/results.json');assert len(rows)==len(old)==60
retained=[];gained=[]
for row in rows:
 prior=old[row['id']];assert row['before']==prior['before']
 if prior['accepted']:
  assert row['accepted'] and row['after']==prior['after'],row['id']
  retained.append(row['id'])
 elif row['accepted']:gained.append(row['id'])
assert len(retained)==6 and gained
assert read('train-sample/report.json')['total']==12
build=(r/'build.log').read_text();tests=sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed',build));assert tests==59,tests
workspace=(r/'workspace.log').read_text();assert "Fixture smoke: {'turns': 4, 'right': 4, 'refused': 0, 'wrong': 0" in workspace
report=dict(candidate=candidate,world_tests=tests,new_native_total=22,new_implementations=16,unchanged_old_native=unchanged,train_total=60,retained=retained,gained=gained,all_pre_dev_gates_passed=True)
names=['build.log','workspace.log','native/report.json','native-verification/results.json','old-initials/report.json','old-native/report.json','independent/report.json','unset-target/report.json','extra/report.json','train-sample/report.json','train-full/report.json','train-full-verification/results.json']
report['evidence']={name:dict(bytes=(r/name).stat().st_size,sha256=sha(r/name)) for name in names}
with (r/'pre-dev.json').open('x') as f:f.write(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(all_pre_dev_gates_passed=True,world_tests=tests,new_implementations=16,train_gained=gained)))
