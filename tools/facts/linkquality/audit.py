#!/usr/bin/env python3
"""Offline train/valid-only exact scorer semantics and counterfactual audit.

Inference input is question-only; this separate process reads annotations only
AFTER prediction. It neither changes the scorer nor provides runtime features.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path


def status(row, answer):
    if len(answer['answers']) == 1 and row[2] in answer['answers']:
        return 'right'
    return 'refused' if answer['refusal'] is not None else 'wrong'


def fingerprint(path):
    return {'path': str(path.resolve()), 'bytes': path.stat().st_size,
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--data', type=Path, required=True)
    p.add_argument('--split', choices=['train', 'valid'], required=True)
    p.add_argument('--pairs', type=Path, required=True)
    p.add_argument('--rows-out', type=Path, required=True)
    p.add_argument('--limit', type=int)
    p.add_argument('--frozen', type=Path)
    a = p.parse_args()
    source = a.data / (a.split + '.tsv')
    rows = [line.split('\t', 3) for line in source.read_text().splitlines()]
    if a.limit is not None:
        rows = rows[:a.limit]
    pairs = [json.loads(line) for line in a.pairs.read_text().splitlines()]
    assert len(rows) == len(pairs), (len(rows), len(pairs))
    if a.frozen:
        frozen = [json.loads(line) for line in a.frozen.read_text().splitlines()]
        assert len(frozen) == len(rows)
        assert all(pair['baseline'] == old for pair, old in zip(pairs, frozen))
    counts = {k: collections.Counter() for k in ('baseline', 'candidate')}
    transitions = collections.Counter()
    scored_right_entity_mismatch = {k: [] for k in ('baseline', 'candidate')}
    answered_entity_correct = collections.Counter()
    lost_right, wrong_removed, link_right_removed = [], [], []
    changed_claims, revived, unchanged, withdrawn = [], [], 0, 0
    audits = []
    for i, (row, pair) in enumerate(zip(rows, pairs), 1):
        old, new = pair['baseline'], pair['candidate']
        before, after = status(row, old), status(row, new)
        for key, value in [('baseline', before), ('candidate', after)]:
            counts[key][value] += 1
        transitions[before + '->' + after] += 1
        for key, prediction in pair.items():
            if prediction['refusal'] is None and prediction['entity'] == row[0]:
                answered_entity_correct[key] += 1
            if status(row, prediction) == 'right' and prediction['entity'] != row[0]:
                scored_right_entity_mismatch[key].append(i)
        identical = old == new
        claim_identical = all(old[k] == new[k] for k in ('entity', 'relation', 'answers'))
        valid_withdrawal = (old['refusal'] is None and bool(old['answers'])
                            and new['refusal'] is not None and not new['answers']
                            and old['entity'] == new['entity']
                            and old['relation'] == new['relation'])
        unchanged += identical
        withdrawn += valid_withdrawal
        if not identical and not valid_withdrawal:
            changed_claims.append(i)
        if old['refusal'] is not None and new != old:
            revived.append(i)
        if before == 'right' and after != 'right':
            lost_right.append(i)
        if before == 'wrong' and after == 'refused':
            wrong_removed.append(i)
        if valid_withdrawal and old['entity'] == row[0]:
            link_right_removed.append(i)
        audits.append({'row': i, 'before_status': before, 'after_status': after,
                       'identical': identical, 'claim_identical': claim_identical,
                       'valid_withdrawal': valid_withdrawal,
                       'before_entity_correct': old['entity'] == row[0],
                       'baseline': old, 'candidate': new})
    a.rows_out.write_text(''.join(json.dumps(row, sort_keys=True) + '\n' for row in audits))
    result = {'split': a.split, 'total': len(rows), 'counts': counts,
              'transitions': transitions, 'unchanged': unchanged, 'withdrawn': withdrawn,
              'answered_entity_correct': answered_entity_correct,
              'scored_right_entity_mismatch': scored_right_entity_mismatch,
              'changed_claims': changed_claims, 'revived_refusals': revived,
              'new_wrong_rows': [r['row'] for r in audits if r['after_status'] == 'wrong'
                                 and r['before_status'] != 'wrong'],
              'correct_to_refused_rows': lost_right, 'wrong_to_refused_rows': wrong_removed,
              'correct_entity_links_removed': link_right_removed,
              'files': [fingerprint(p) for p in (source, a.pairs, a.rows_out)]}
    print(json.dumps(result, indent=2))
    assert not changed_claims and not revived and not result['new_wrong_rows']


if __name__ == '__main__':
    main()
