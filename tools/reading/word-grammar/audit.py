#!/usr/bin/env python3
"""Offline replay of saved costs only; never calls either model or changes rows."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--input', required=True, type=Path)
p.add_argument('--scores', required=True, type=Path)
p.add_argument('--baseline-report', required=True, type=Path)
p.add_argument('--output', required=True, type=Path)
p.add_argument('--full', action='store_true')
a = p.parse_args()
def digest(data): return hashlib.sha256(data).hexdigest()
def record(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': digest(data)}
def pref(g, b): return 'tie' if abs(g-b) <= 1e-9 else 'good' if g < b else 'bad'
def totals(rows):
    b = Counter(r['baseline_choice'] for r in rows)
    c = Counter(r['candidate_choice'] for r in rows)
    return {'pairs': len(rows), 'baseline': {k: b[k] for k in choices},
            'candidate': {k: c[k] for k in choices},
            'transitions': [[sum(r['baseline_choice'] == x and r['candidate_choice'] == y for r in rows)
                             for y in choices] for x in choices]}
choices = ['good', 'bad', 'tie']
raw = a.input.read_bytes().splitlines(keepends=True)
scores = [json.loads(x) for x in a.scores.read_text().splitlines()]
assert len(raw) == len(scores) == (1340 if a.full else 20)
for i, (line, score) in enumerate(zip(raw, scores), 1):
    row = json.loads(line)
    assert score['row'] == i and score['group'] == row['group'] and score['id'] == row.get('id')
    assert score['baseline_choice'] == pref(score['baseline_good_bits'], score['baseline_bad_bits'])
    assert score['candidate_choice'] == pref(score['candidate_good_token_bits'], score['candidate_bad_token_bits'])
    score['input_row_sha256'] = digest(line)
groups = {g: totals([r for r in scores if r['group'] == g]) for g in sorted({r['group'] for r in scores})}
result = {'input': record(a.input), 'raw_scores': record(a.scores), 'total': totals(scores), 'groups': groups,
          'transition_order': choices, 'baseline_regressions': [r for r in scores if r['baseline_choice'] == 'good' and r['candidate_choice'] != 'good'],
          'policy': 'original row order; total costs including EOS; tie abs(diff)<=1e-9; word token cost is NOT byte likelihood'}
baseline = json.loads(a.baseline_report.read_text())
assert len(groups) == (67 if a.full else 1)
for group, stats in groups.items():
    retained = baseline['groups'][group]
    assert stats['pairs'] == retained['pairs']
    assert stats['baseline'] == {k: retained['sixgram']['ties' if k == 'tie' else k] for k in choices}
if a.full:
    assert result['total']['baseline'] == {'good': 707, 'bad': 633, 'tie': 0}
    result['accepted'] = result['total']['candidate']['good'] >= 747
    result['acceptance'] = 'at least 747 good preferences / exactly 1340; one candidate, no retuning'
    result['group_changes'] = dict(Counter('improved' if v['candidate']['good'] > v['baseline']['good'] else 'regressed' if v['candidate']['good'] < v['baseline']['good'] else 'equal' for v in groups.values()))
a.output.mkdir(parents=True, exist_ok=True)
with (a.output/'rows.jsonl').open('x') as f:
    for row in scores: f.write(json.dumps(row, sort_keys=True) + '\n')
with (a.output/'report.json').open('x') as f: f.write(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['total', 'accepted', 'group_changes'] if k in result}, indent=2))
