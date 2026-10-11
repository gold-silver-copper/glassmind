#!/usr/bin/env python3
"""Offline annotation audit, separate from the unchanged answer-only scorer.

Runtime receives questions only. This tool joins already-frozen predictions to
TRAIN/valid annotations and requires their exact question order and row count.
Joint agreement is not a proof of global uniqueness or current factual truth.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def identity(path):
    raw = path.read_bytes()
    return dict(path=str(path.resolve()), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())


def audit(rows, predictions):
    if len(rows) != len(predictions) or not rows:
        raise ValueError('nonempty, complete prediction coverage required')
    answer_counts, joint_counts = Counter(), Counter()
    coincidences = []
    for number, (row, pred) in enumerate(zip(rows, predictions, strict=True), 1):
        subject, relation, obj, _ = row
        if set(pred) != {'entity', 'relation', 'answers', 'refusal'}:
            raise ValueError('unexpected prediction schema')
        answers = pred['answers']
        if not isinstance(answers, list) or any(not isinstance(x, str) for x in answers):
            raise ValueError('answer IDs must be a list of strings')
        if len(answers) > 1 or bool(answers) == (pred['refusal'] is not None):
            raise ValueError('expected one answer or an explicit empty-answer refusal')
        answer_match = answers == [obj]
        entity_match = pred['entity'] == subject
        relation_match = pred['relation'] == relation
        answer_status = 'right' if answer_match else ('refused' if pred['refusal'] is not None else 'wrong')
        joint_status = 'right' if answer_match and entity_match and relation_match else ('refused' if pred['refusal'] is not None else 'wrong')
        answer_counts[answer_status] += 1
        joint_counts[joint_status] += 1
        if answer_match and joint_status != 'right':
            coincidences.append(dict(row=number, entity_matches=entity_match,
                                     relation_matches=relation_match,
                                     gold=dict(entity=subject, relation=relation, answer=obj),
                                     predicted=pred))
    return dict(total=len(rows), answer_only={k:answer_counts[k] for k in ('right','refused','wrong')},
                joint_entity_relation_answer={k:joint_counts[k] for k in ('right','refused','wrong')},
                answer_matches_with_wrong_link_or_relation=coincidences)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--labels', type=Path, required=True)
    p.add_argument('--questions', type=Path, required=True)
    p.add_argument('--predictions', type=Path, required=True)
    p.add_argument('--official-totals', type=Path, required=True)
    p.add_argument('--scope', required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    if a.labels.name not in ('train.tsv', 'valid.tsv'):
        p.error('only existing public TRAIN/validation annotations are permitted')
    rows = [line.split('\t', 3) for line in a.labels.read_text().splitlines()]
    if any(len(row) != 4 for row in rows):
        p.error('malformed annotation row')
    if a.questions.read_text() != ''.join(row[3]+'\n' for row in rows):
        p.error('question input does not match the complete annotation order')
    predictions = [json.loads(line) for line in a.predictions.read_text().splitlines()]
    result = audit(rows, predictions)
    official = json.loads(a.official_totals.read_text())
    if official['total'] != result['total'] or any(official[k] != v for k,v in result['answer_only'].items()):
        p.error('answer-only reconstruction disagrees with frozen authoritative totals')
    result.update(metric='joint_annotated_entity_relation_answer', scope=a.scope,
                  artifacts=[identity(x) for x in (a.labels,a.questions,a.predictions,a.official_totals,Path(__file__))],
                  limitations='Separate diagnostic; original answer-only scorer and candidate acceptance criteria are unchanged. Exact annotated IDs, no redirect normalization. Dataset errors, snapshot differences, omitted question constraints and incomplete inverse uniqueness remain possible.')
    a.output.parent.mkdir(parents=True, exist_ok=True)
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ('scope','total','answer_only','joint_entity_relation_answer')}))


if __name__ == '__main__':
    main()
