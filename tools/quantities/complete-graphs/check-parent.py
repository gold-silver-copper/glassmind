"""Apply frozen question-only expectations to untouched teacher-compiled graphs."""
from fractions import Fraction
from pathlib import Path
import hashlib,json,subprocess
ROOT=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/quantity-graphs')
freeze=json.loads((ROOT/'parent/freeze.json').read_text())
for name,digest in freeze['files'].items():
    assert hashlib.sha256((ROOT/'parent'/name).read_bytes()).hexdigest()==digest
rows={r['id']:r for r in json.loads((ROOT/'source/batch.json').read_text())['rows']}
actual={r['id']:r for r in json.loads((ROOT/'mechanical-report.json').read_text())['rows']}
assert set(rows)==set(actual) and len(actual)==32
cases=[];requests=[];checks=[]
for expected in json.loads((ROOT/'parent/expectations.json').read_text()):
    key=expected['id'];a=actual[key];q=rows[key]
    assert hashlib.sha256(q['question'].encode()).hexdigest()==expected['question_sha256']
    if expected.get('reject'):
        agrees=a['status'] in expected['allowed_statuses'] and a['program'] is None
    else:
        agrees=a['status']=='resolved' and a['program'] is not None
        if agrees:
            requests.append(dict(question=q['question'],program=a['program']))
            checks.append(dict(id=key,kind='original',expected=expected['expected']))
            for mutation in expected['mutations']:
                changes={int(i):v for i,v in mutation['changes'].items()}
                # A frozen mutation may name one representative of a coreferential
                # group. Change all of its mentions together, preserving identity.
                for mentions in a['quantity_mentions']:
                    values={v for i,v in changes.items() if i in mentions}
                    assert len(values)<=1
                    if values:
                        value=next(iter(values));changes.update({i:value for i in mentions})
                raw=q['question'].encode()
                for i,value in sorted(changes.items(),key=lambda item:q['numbers'][item[0]]['start'],reverse=True):
                    n=q['numbers'][i];raw=raw[:n['start']]+value.encode()+raw[n['end']:]
                requests.append(dict(question=raw.decode(),program=a['program']))
                checks.append(dict(id=key,kind='source_counterfactual',changes=changes,expected=mutation['expected']))
    cases.append(dict(id=key,status=a['status'],status_agrees=agrees,required_rejection=expected.get('reject',False)))
executor=ROOT.parent.parent/'groundedsteps-source/equation_executor'
assert hashlib.sha256(executor.read_bytes()).hexdigest()=='40e6a6e975cf8474231ac0e3bcc2e3df5c1ce8196dead0b9159586aa5935ea6f'
p=subprocess.run([str(executor)],input=''.join(json.dumps(x)+'\n' for x in requests),text=True,capture_output=True,check=True,timeout=30)
outputs=[json.loads(line) for line in p.stdout.splitlines()];assert len(outputs)==len(checks)
for request,c,o in zip(requests,checks,outputs,strict=True):
    c['matches']=o['status']=='proposed' and Fraction(o['value'])==Fraction(c['expected']) and o['program']==request['program']
for case in cases:
    case['agrees']=case['status_agrees'] and all(c['matches'] for c in checks if c['id']==case['id'])
report=dict(cases=cases,all_eight_agree=all(c['agrees'] for c in cases),native_requests=len(requests),all_original_and_counterfactual_values_match=all(c['matches'] for c in checks),checks=checks,outputs=outputs,requests=requests,all_32_semantic_review_still_required=True)
with (ROOT/'parent-comparison.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in report.items() if k not in ('checks','outputs','requests')}))
if not report['all_eight_agree']:raise SystemExit(1)
