#!/usr/bin/env python3
"""Offline coverage/overlap diagnostics only; TRAIN and valid are explicit."""
import collections
import json
import re
import sys
from pathlib import Path

run, data = map(Path, sys.argv[1:])
train = [l.split('\t', 3) for l in (data / 'train.tsv').read_text().splitlines()]
valid = [l.split('\t', 3) for l in (data / 'valid.tsv').read_text().splitlines()]

def normalized(text):
    return ' '.join(re.findall(r'[^\W_]+', text.lower()))


def matches(index, row, prediction):
    if prediction['entity'] is None:
        return set()
    question = ' ' + normalized(row[3]) + ' '
    aliases = [normalized(a) for a in index[prediction['entity']]['aliases']]
    aliases = [a for a in aliases if len(a.encode()) >= 2 and ' ' + a + ' ' in question]
    longest = max((len(a.encode()) for a in aliases), default=0)
    return {a for a in aliases if len(a.encode()) == longest}

report = {'selection': 'First floor(0.9 * 19481)=17532 TRAIN rows minus zero-based i%10==0; frozen top1000 selected by frequency and SHA256 tie-break, one redirect excluded, actual999. No new entity selection in this batch.'}
for scope, path in [('old500', Path('tools/facts/index.json')),
                    ('new999', run / 'baseline/train-index.json')]:
    index = json.loads(path.read_text())['entities']
    train_p = [json.loads(l) for l in (run / 'artifacts' / (scope + '-train-base.jsonl')).read_text().splitlines()]
    valid_p = [json.loads(l) for l in (run / 'artifacts' / (scope + '-valid-frozen.jsonl')).read_text().splitlines()]
    sets = {}
    totals = {}
    for split, numbered in [('fit', [(i, r, p) for i, (r, p) in enumerate(zip(train, train_p)) if i % 10 != 0]),
                            ('cal', [(i, r, p) for i, (r, p) in enumerate(zip(train, train_p)) if i % 10 == 0]),
                            ('selection', [(i, r, p) for i, (r, p) in enumerate(zip(train, train_p)) if i < len(train) * 9 // 10 and i % 10 != 0]),
                            ('valid', [(i, r, p) for i, (r, p) in enumerate(zip(valid, valid_p))])]:
        subjects = {r[0] for _, r, _ in numbered}
        questions = {normalized(r[3]) for _, r, _ in numbered}
        phrases = set().union(*(matches(index, r, p) for _, r, p in numbered))
        sets[split] = dict(subjects=subjects, questions=questions, linked_alias_phrases=phrases)
        totals[split] = {'rows': len(numbered), 'unique_subjects': len(subjects),
                         'rows_subject_in_index': sum(r[0] in index for _, r, _ in numbered),
                         'answered': sum(p['refusal'] is None for _, _, p in numbered),
                         'answered_entity_correct': sum(p['refusal'] is None and p['entity'] == r[0] for _, r, p in numbered)}
    overlaps = {}
    for left, right in [('fit', 'cal'), ('selection', 'cal'), ('fit', 'valid'), ('selection', 'valid'), ('cal', 'valid')]:
        overlaps[left + '/' + right] = {
            kind: {'left_unique': len(sets[left][kind]), 'right_unique': len(sets[right][kind]),
                   'intersection_unique': len(sets[left][kind] & sets[right][kind])}
            for kind in sets[left]
        }
    report[scope] = {'index_entities': len(index), 'partitions': totals, 'overlaps': overlaps}
print(json.dumps(report, indent=2))
