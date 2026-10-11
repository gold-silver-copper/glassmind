from pathlib import Path
from fractions import Fraction
import argparse, hashlib, json, subprocess
p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
here = Path(__file__).resolve().parent
for name, sha in json.loads((here/'freeze.json').read_text()).items():
    assert hashlib.sha256((here/name).read_bytes()).hexdigest() == sha
cases = json.loads((here/'cases.json').read_text())
r = subprocess.run([str(a.binary.resolve()), '--jsonl'], input=''.join(json.dumps(c['question'])+'\n' for c in cases), text=True, capture_output=True, check=True, timeout=60)
answers = [json.loads(line) for line in r.stdout.splitlines()]
for c, answer in zip(cases, answers, strict=True):
    c['actual'] = answer
    c['passed'] = (answer['status']=='refusal' if c['expected'] is None else answer['status']=='answer' and answer.get('unit') is None and Fraction(answer['value'])==Fraction(c['expected']))
report = dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), total=len(cases), passed=sum(c['passed'] for c in cases), cases=cases)
a.out.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
for c in cases:
    if not c['passed']: print(c['question'], c['expected'], c['actual'])
