#!/usr/bin/env python3
"""Public authored comparison review and question-only legacy byte compatibility.
No official test split, labels, hidden generators, or model calls are used.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
from fractions import Fraction

p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--typed', type=Path, required=True)
p.add_argument('--source', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
a.out.mkdir(parents=True, exist_ok=True)

def digest(data): return hashlib.sha256(data).hexdigest()
def run(binary, mode, data, name):
    proc = subprocess.run([str(binary), *mode], input=data, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, timeout=120)
    (a.out / (name + '.jsonl')).write_bytes(proc.stdout)
    (a.out / (name + '.stderr')).write_bytes(proc.stderr)
    assert proc.returncode == 0, (name, proc.returncode)
    return proc.stdout

# Fixed sample MUST precede full affected dev and authored synthetic compatibility.
parity = {}
manifest = json.loads((a.source / 'manifest.json').read_text())
for split, count in [('sample', 200), ('full', 4475), ('synthetic', 3116)]:
    names = [split + '-questions.jsonl', split + '-baseline.jsonl']
    for name in names:
        data = (a.source / name).read_bytes()
        assert digest(data) == manifest['files'][name]['sha256'], name
        assert len(data) == manifest['files'][name]['bytes'], name
    questions = (a.source / names[0]).read_bytes()
    baseline = (a.source / names[1]).read_bytes()
    candidate = run(a.binary, ['--jsonl'], questions, split + '-candidate')
    assert len(questions.splitlines()) == count
    assert candidate == baseline, split + ' legacy byte mismatch'
    parity[split] = dict(rows=count, byte_identical=True, bytes=len(candidate), sha256=digest(candidate))

cases = json.loads((Path(__file__).parent / 'authored.json').read_text())
questions = ''.join(json.dumps(c['question']) + '\n' for c in cases).encode()
(a.out / 'authored-input.jsonl').write_bytes(questions)
raw = run(a.binary, ['--compare-jsonl'], questions, 'authored-candidate')
results = [json.loads(s) for s in raw.splitlines()]
assert len(results) == len(cases)
direct = []
parsed = []
for case, response in zip(cases, results):
    if 'refusal' in case:
        assert response['status'] == 'refusal', (case, response)
        assert response['error']['kind'] == case['refusal'], (case, response)
        continue
    assert response['status'] == 'comparison', (case, response)
    result = response['result']
    for field in ['dimension', 'ordering']:
        assert result[field] == case[field], (case, result)
    assert result['witness']['left'] == case['witness_left'], (case, result)
    assert result['witness']['right'] == case['witness_right'], (case, result)
    # Independent arbitrary precision witness ordering (no unit scale duplication).
    l, r = [Fraction(result['witness'][side]) for side in ['left', 'right']]
    assert result['ordering'] == ('less' if l < r else 'greater' if l > r else 'equal')
    assert result['outcome']['kind'] == case['kind']
    op, val = ('predicate', 'truth') if case['kind'] == 'predicate' else ('selection', 'selected')
    assert result['outcome'][op] == case['operation'], (case, result)
    assert result['outcome'][val] == case['answer'], (case, result)
    # Independently split original questions for direct API calls, preserving units.
    question = re.sub(r'\s+', ' ', case['question'].strip()).lower().removesuffix('?').strip()
    if case['kind'] == 'predicate':
        match = re.fullmatch(r'is (.+) (?:greater than|less than|equal to|at least|at most) (.+)', question)
    else:
        match = re.fullmatch(r'which is [a-z]+\s*,\s*(.+) or (.+)', question)
    assert match
    operands = []
    for operand in match.groups():
        value, unit = operand.split(' ', 1)
        operands.append(dict(value=value, unit=unit))
    direct.append(dict(left=operands[0], right=operands[1], operation=dict(kind=case['kind'], operation=case['operation'])))
    parsed.append(response)
raw_direct = ''.join(json.dumps(d)+'\n' for d in direct).encode()
(a.out / 'typed-input.jsonl').write_bytes(raw_direct)
typed = run(a.typed, [], raw_direct, 'typed-candidate')
assert [json.loads(s) for s in typed.splitlines()] == parsed

# Transport must refuse and continue, not truncate an oversized prefix to a valid query.
raw_invalid = (b'"Is 1 kg greater than 1 g?' + b' ' * 100000 + b'"\n' +
               b'{}\nnull\n"Is 1 m equal to 1 m?\\u0000"\n"Is 1 m equal to 100 cm?"\n')
transport = run(a.binary, ['--compare-jsonl'], raw_invalid, 'transport')
t = [json.loads(s) for s in transport.splitlines()]
assert [x['status'] for x in t] == ['refusal'] * 4 + ['comparison']
report = dict(legacy_compatibility=parity, authored=dict(total=len(cases), comparisons=len(parsed),
              expected_refusals=len(cases)-len(parsed), wrong=0, direct_parsed_equal=len(parsed)),
              transport_checks=len(t), scope='Authored literal comparisons only; no new MathQA accuracy measurement.',
              binary_sha256=digest(a.binary.read_bytes()), typed_sha256=digest(a.typed.read_bytes()))
(a.out / 'report.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report, indent=2))
