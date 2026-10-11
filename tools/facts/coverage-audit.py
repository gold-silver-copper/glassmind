#!/usr/bin/env python3
"""Offline public TRAIN/validation snapshot diagnostic; never runtime answers.

Gold entity/relation IDs identify missing knowledge, not proposed predictions.
Even unique matching claims in a finite snapshot need not be globally unique.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def identity(path):
    raw = path.read_bytes()
    return dict(path=str(path.resolve()), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('index', 'labels', 'questions', 'predictions', 'output'):
        p.add_argument('--'+name, type=Path, required=True)
    p.add_argument('--paired', action='store_true', help='Read baseline from paired frozen predictions')
    a = p.parse_args()
    if a.labels.name not in ('train.tsv', 'valid.tsv'):
        p.error('only public TRAIN/validation annotations are permitted')
    rows = [s.split('\t', 3) for s in a.labels.read_text().splitlines()]
    if not rows or any(len(r) != 4 for r in rows):
        p.error('nonempty complete annotations required')
    if a.questions.read_text() != ''.join(r[3]+'\n' for r in rows):
        p.error('question input differs from complete annotation order')
    predictions = [json.loads(s) for s in a.predictions.read_text().splitlines()]
    if a.paired:
        predictions = [r['baseline'] for r in predictions]
    if len(predictions) != len(rows):
        p.error('incomplete predictions')
    entities = json.loads(a.index.read_text())['entities']
    counts, directions, claims, outcomes = Counter(), Counter(), Counter(), Counter()
    for (entity, relation, answer, _), pred in zip(rows, predictions, strict=True):
        counts['total'] += 1
        if entity not in entities:
            continue
        counts['annotated_entity_in_index'] += 1
        directions[relation[:1]] += 1
        if relation.startswith('P'):
            values = set(entities[entity]['claims'].get(relation, []))
        elif relation.startswith('R'):
            values = {e for e,row in entities.items()
                      if entity in row['claims'].get('P'+relation[1:], [])}
        else:
            p.error('unknown relation direction')
        outcome = ('missing' if not values else 'unique_gold' if values == {answer}
                   else 'nonunique_including_gold' if answer in values else 'gold_absent_from_nonempty')
        claims[outcome] += 1
        if pred['entity'] == entity:
            counts['entity_link_correct'] += 1
            if pred['relation'] == relation:
                counts['entity_and_relation_correct'] += 1
                outcomes[pred['refusal'] or 'answered'] += 1
    result = dict(counts=dict(counts), annotated_relation_directions=dict(directions),
                  gold_relation_claim_diagnostic=dict(claims), correct_link_and_relation_outcomes=dict(outcomes),
                  artifacts=[identity(f) for f in (a.index, a.labels, a.questions, a.predictions, Path(__file__))],
                  note='Gold entity/relation used only to diagnose snapshot coverage; not model predictions, attainable accuracy or global inverse uniqueness. No runtime/index/scorer changes or inference.')
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('artifacts','note')}))


if __name__ == '__main__':
    main()
