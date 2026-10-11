#!/usr/bin/env python3
"""Compare frozen dev outputs only; no generation, gold adjustment or solver."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
out = Path(os.environ['TMPDIR'])
base = out / 'glassmind-quantities-baseline'
candidate = Path(os.environ.get('CANDIDATE_BINARY', str(Path(os.environ['CARGO_TARGET_DIR']) / 'debug/glassmind-quantities')))

def digest(data):
    return hashlib.sha256(data).hexdigest()

def synthetic(binary, label):
    rows = []
    for name in ('synthetic-dev', 'extended-dev'):
        rows.extend(json.loads(line) for line in (root / f'crates/quantities/dev/{name}.jsonl').read_text().splitlines())
    assert len(rows) == 3116
    inputs = ''.join(json.dumps(row['question']) + '\n' for row in rows)
    run = subprocess.run([str(binary), '--jsonl'], input=inputs, text=True,
                         capture_output=True, check=True, timeout=120)
    (out / f'{label}-synthetic-predictions.jsonl').write_text(run.stdout)
    answers = [json.loads(line) for line in run.stdout.splitlines()]
    assert len(answers) == len(rows)
    statuses = [(a['status'], a.get('value'), a.get('unit')) for a in answers]
    return statuses, digest(json.dumps(statuses).encode())

old, old_hash = synthetic(base, 'baseline')
new, new_hash = synthetic(candidate, 'candidate')
changed = [i for i, (a, b) in enumerate(zip(old, new)) if a != b]
b = json.loads((out / 'baseline-full.json').read_text())
c = json.loads((out / 'candidate-full.json').read_text())
old_answers = {a['index']: a for a in b['answered']}
new_answers = {a['index']: a for a in c['answered']}
new_cases = [a for i, a in new_answers.items() if i not in old_answers]
lost_correct = [i for i, a in old_answers.items() if a['outcome'] == 'right' and new_answers.get(i, {}).get('outcome') != 'right']
changed_answers = [i for i, a in old_answers.items() if new_answers.get(i) != a]
summary = dict(synthetic_total=len(old), synthetic_changed=changed,
               baseline_synthetic_status_value_unit_sha256=old_hash,
               candidate_synthetic_status_value_unit_sha256=new_hash,
               lost_correct=lost_correct, changed_baseline_answers=changed_answers,
               new_answers=new_cases,
               baseline={k: v for k, v in b.items() if k != 'answered'},
               candidate={k: v for k, v in c.items() if k != 'answered'})
(out / 'comparison.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))
assert not changed and not lost_correct and not changed_answers
