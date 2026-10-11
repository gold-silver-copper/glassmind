#!/usr/bin/env python3
"""Independent development checks for absolute differences and ambiguous targets.

Enumerate a range containing both real roots of |x-c|=d for the chosen integer
c,d. This oracle does not use affine elimination or candidate parsing.
"""
import argparse
import json
import subprocess
from fractions import Fraction
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
cases = []
for center in [-6, -2, 0, 3, 6]:
    for distance in [-1, 0, 1, 4]:
        solutions = [x for x in range(-20, 21) if abs(x - center) == distance]
        statement = f'The difference between a number and {center} is {distance}. '
        for question, values in [
            ('What is the number?', set(solutions)),
            ('What is the difference between the number and 0?', {abs(x) for x in solutions}),
        ]:
            expected = str(next(iter(values))) if len(values) == 1 else None
            cases.append(dict(question=statement + question, expected=expected))
for x, y in [(-3, 7), (7, -3), (2, 3), (3, 2), (0, 0)]:
    prefix = f'The first number is {x}. The second number is {y}. '
    cases.append(dict(question=prefix + 'What is twice the difference between the first number and the second number?', expected=str(2*abs(x-y))))
    cases.append(dict(question=prefix + 'What is the first number minus the second number?', expected=str(x-y)))
    cases.append(dict(question=prefix + 'What is the difference between the difference between the first number and the second number and 4?', expected=str(abs(abs(x-y)-4))))
result = subprocess.run([str(a.binary), '--jsonl'], input=''.join(json.dumps(c['question'])+'\n' for c in cases), capture_output=True, text=True, timeout=120, check=True)
answers = [json.loads(line) for line in result.stdout.splitlines()]
assert len(cases) == len(answers)
for case, answer in zip(cases, answers, strict=True):
    expected = case['expected']
    case['actual'] = answer
    case['passed'] = (answer['status'] == 'refusal' if expected is None else
                      answer['status'] == 'answer' and Fraction(answer['value']) == Fraction(expected) and answer.get('unit') is None)
report = dict(total=len(cases), right=sum(c['passed'] for c in cases), cases=cases,
              source='independent finite-root enumeration and direct arithmetic; local development only', license='MIT OR Apache-2.0')
a.output.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({k: v for k, v in report.items() if k != 'cases'}))
assert report['total'] == report['right']
