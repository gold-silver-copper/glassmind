"""Independent affine range oracle on rectangles, expressed via absolute values."""
from pathlib import Path
from fractions import Fraction
import itertools, hashlib, json, subprocess
HERE = Path(__file__).resolve().parent
cases = []
for (lx, ux, ly, uy), (a,b) in itertools.product([
    (-3,7,-2,8), (-3,7,4,4), (2,2,-1,5), (2,2,-1,-1),
    (3,2,-1,5), (-2,4,3,1), (-3,-3,-7,-7),
], itertools.product([-2,0,3],[-1,0,4])):
    # |x-L|=x-L iff x>=L; |x-U|=U-x iff x<=U.
    prefix = (f'The difference between the first number and {lx} is the first number minus {lx}. '
              f'The difference between the first number and {ux} is {ux} minus the first number. '
              f'The difference between the second number and {ly} is the second number minus {ly}. '
              f'The difference between the second number and {uy} is {uy} minus the second number. ')
    values = {a*x+b*y for x,y in itertools.product([lx,ux],[ly,uy])} if lx<=ux and ly<=uy else set()
    expected = str(next(iter(values))) if len(values)==1 else None
    cases.append(dict(question=prefix+f'What is {a} times the first number plus {b} times the second number?',
                      bounds=[lx,ux,ly,uy], coefficients=[a,b], expected=expected))
raw = json.dumps(cases,indent=2)+'\n'
(HERE/'rectangle-cases.json').write_text(raw)
(HERE/'rectangle-freeze.json').write_text(json.dumps(dict(cases_sha256=hashlib.sha256(raw.encode()).hexdigest(),
    oracle='Affine function extrema on a nonempty closed rectangle occur at corners; reversed bounds mean empty feasible set.'),indent=2)+'\n')
r = subprocess.run([str(HERE/'candidate'),'--jsonl'], input=''.join(json.dumps(c['question'])+'\n' for c in cases),
                   capture_output=True,text=True,check=True,timeout=60)
for case, actual in zip(cases,[json.loads(x) for x in r.stdout.splitlines()],strict=True):
    case['actual'] = actual
    case['passed'] = (actual['status']=='refusal' if case['expected'] is None else
                      actual['status']=='answer' and actual.get('unit') is None and Fraction(actual['value'])==Fraction(case['expected']))
report = dict(total=len(cases),passed=sum(c['passed'] for c in cases),cases=cases)
(HERE/'rectangles.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
assert all(c['passed'] for c in cases), [c for c in cases if not c['passed']]
