"""Compare every prospective case without correcting a teacher's output."""
from pathlib import Path
import hashlib,json,subprocess,sys
ROOT=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/span-expansion')
sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'span-references'))
from validate import canonical
freeze=json.loads((ROOT/'parent/freeze.json').read_text())
for name,digest in freeze['files'].items():
 assert hashlib.sha256((ROOT/'parent'/name).read_bytes()).hexdigest()==digest
rows={r['id']:r for r in json.loads((ROOT/'source/batch.json').read_text())['rows']}
actual={}
for i in range(1,5):
 report=json.loads((ROOT/f'batch-{i:02d}/mechanical-report.json').read_text())
 for row in report['rows']:
  assert row['id'] not in actual
  actual[row['id']]=row
assert set(rows)==set(actual) and len(actual)==128
cases=[];requests=[];checks=[]
for expected in json.loads((ROOT/'parent/expectations.json').read_text()):
 key=expected['id'];a=actual[key];q=rows[key];agrees=True
 if expected.get('reject'):
  agrees=a['status'] in expected['allowed_statuses'] and not a['programs']
 else:
  agrees=a['status'] in ('resolved','equivalent') and bool(a['programs'])
  for p in a['programs']:
   same=canonical(p['steps'])==canonical(expected['program']['steps'])
   agrees=agrees and same
   if not same:continue
   requests.append(dict(question=q['question'],program=p));checks.append(dict(id=key,kind='original',expected=q['final_answer']))
   for index,value,result in expected['mutations']:
    n=q['numbers'][index];raw=q['question'].encode();changed=(raw[:n['start']]+value.encode()+raw[n['end']:]).decode()
    requests.append(dict(question=changed,program=p));checks.append(dict(id=key,kind='source_counterfactual',input=index,replacement=value,expected=result))
 cases.append(dict(id=key,status=a['status'],agrees=agrees,required_rejection=expected.get('reject',False),semantic_reference_frozen=True))
executor=ROOT.parent.parent/'groundedsteps-source/equation_executor'
p=subprocess.run([str(executor)],input=''.join(json.dumps(x)+'\n' for x in requests),text=True,capture_output=True,check=True,timeout=30)
outputs=[json.loads(line) for line in p.stdout.splitlines()];assert len(outputs)==len(checks)
for c,o in zip(checks,outputs,strict=True):c['matches']=o['status']=='proposed' and o['value']==c['expected']
report=dict(cases=cases,all_eight_agree=all(c['agrees'] for c in cases),native_requests=len(requests),all_original_and_counterfactual_values_match=all(c['matches'] for c in checks),checks=checks,outputs=outputs,all_128_semantic_review_still_required=True)
with (ROOT/'parent-comparison.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in report.items() if k not in ('checks','outputs')}))
if not report['all_eight_agree'] or not report['all_original_and_counterfactual_values_match']:raise SystemExit(1)
