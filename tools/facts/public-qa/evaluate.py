#!/usr/bin/env python3
"""Frozen CAL comparison; runtime sees questions, annotation joins stay offline."""
from collections import Counter
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import time

RUN = Path(__file__).resolve().parent
WORK = RUN.parent.parent
REPO = WORK / 'wikidata-qa'
GRAPH = RUN.parent / 'wikidata-native/native.sqlite'
MODEL = WORK / 'trainindex-run/tmp/trainindex/bundle1000/relation.json'
TRAIN = Path('/home/gsc/.cache/glassmind/data/simplequestions/train.tsv')
QUESTIONS = WORK / 'linkquality-run/artifacts/train.questions'
BASELINE = WORK / 'linkquality-run/artifacts/new999-train-base.jsonl'
INDEX = WORK / 'trainindex-run/tmp/trainindex/bundle1000/index.json'
BINARY = Path('/home/gsc/code/glassmind/target/release/glassmind-facts')
PINS = [
    (GRAPH, 'f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b'),
    (MODEL, 'db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6'),
    (TRAIN, '25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b'),
    (QUESTIONS, 'd3cd1c39d916195757a304ce254a19e5f3b6b6b7fa19f6356cddbf9fff675eaf'),
    (BASELINE, 'eb09e5fc44e699e2ab08b94a44409ab8ad5a623224bd7340372c4326bcb0aee8'),
    (INDEX, '142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791'),
    (BINARY, 'd733e1d9f19e5e4e5a03087f9d12725fed1b212f5a1b3b47f4573a7ac28d4a3d'),
]


def identity(path):
    with path.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': digest}


def status(row, prediction):
    entity, relation, answer, _ = row
    if prediction['refusal'] is not None:
        return 'refused'
    return 'right' if prediction['entity'] == entity and prediction['relation'] == relation and prediction['answers'] == [answer] else 'wrong'


def native_knowledge(db, rows, predictions):
    counts = Counter()
    categories = Counter()
    directions = Counter()
    for (entity, relation, answer, _), pred in zip(rows, predictions, strict=True):
        counts['total'] += 1
        present = db.execute('SELECT 1 FROM entities WHERE id=?', (entity,)).fetchone() is not None
        counts['annotated_entity_present'] += present
        counts['annotated_entity_has_eligible_alias'] += db.execute('SELECT 1 FROM aliases WHERE entity=? LIMIT 1', (entity,)).fetchone() is not None
        counts['entity_link_correct'] += pred['entity'] == entity
        counts['entity_and_relation_correct'] += pred['entity'] == entity and pred['relation'] == relation
        if not present:
            continue
        directions[relation[0]] += 1
        if relation.startswith('P'):
            values = [r[0] for r in db.execute('SELECT object FROM edges WHERE subject=? AND property=? LIMIT 2', (entity, relation))]
            triple = entity, relation, answer
        elif relation.startswith('R'):
            prop = 'P' + relation[1:]
            values = [r[0] for r in db.execute('SELECT subject FROM edges WHERE property=? AND object=? LIMIT 2', (prop, entity))]
            triple = answer, prop, entity
        else:
            raise ValueError('unsupported annotated relation')
        has_gold = db.execute('SELECT 1 FROM edges WHERE subject=? AND property=? AND object=?', triple).fetchone() is not None
        category = 'missing' if not values else 'unique_gold' if values == [answer] else 'nonunique_including_gold' if has_gold else 'gold_absent_from_nonempty'
        categories[category] += 1
    return {'counts': dict(counts), 'present_anchor_directions': dict(directions), 'local_claim_categories_for_present_anchors': dict(categories)}


def main():
    started = time.monotonic()
    if (RUN / 'predictor').exists() or (RUN / 'cal-predictions.jsonl').exists():
        raise RuntimeError('preserve an existing run; do not overwrite or resume blindly')
    identities = []
    for path, expected in PINS:
        record = identity(path)
        assert record['sha256'] == expected, str(path)
        identities.append(record)
    (RUN / 'inputs.json').write_text(json.dumps(identities, indent=2) + '\n')
    rows = [line.split('\t', 3) for line in TRAIN.read_text().splitlines()]
    assert len(rows) == 19481 and all(len(row) == 4 for row in rows)
    assert QUESTIONS.read_text() == ''.join(row[3] + '\n' for row in rows)
    selected = list(range(0, len(rows), 10))
    cal = [rows[i] for i in selected]
    assert len(cal) == 1949
    question_path = RUN / 'cal-questions.txt'
    question_path.write_text(''.join(row[3] + '\n' for row in cal))
    (RUN / 'cal-indices.json').write_text(json.dumps(selected) + '\n')
    shutil.copy2(BINARY, RUN / 'predictor')
    command = [str(RUN / 'predictor'), 'predict-disk', '--model', str(MODEL), '--database', str(GRAPH)]
    env = {k: v for k, v in os.environ.items() if not k.startswith(('OPENAI', 'ANTHROPIC', 'CLAUDE', 'GEMINI')) and 'API_KEY' not in k}
    inference_started = time.monotonic()
    with question_path.open('rb') as questions, (RUN / 'cal-predictions.jsonl').open('xb') as predictions, (RUN / 'inference.err').open('wb') as errors:
        result = subprocess.run(command, stdin=questions, stdout=predictions, stderr=errors, env=env, timeout=120)
    inference = {'command': command, 'seconds': time.monotonic() - inference_started, 'returncode': result.returncode,
                 'questions': identity(question_path), 'predictions': identity(RUN / 'cal-predictions.jsonl')}
    (RUN / 'inference.json').write_text(json.dumps(inference, indent=2) + '\n')
    assert result.returncode == 0
    predictions = [json.loads(line) for line in (RUN / 'cal-predictions.jsonl').read_text().splitlines()]
    # Only after outputs have been frozen and hashed do we load reference
    # predictions and join either predictor to the calibration annotations.
    baseline_all = [json.loads(line) for line in BASELINE.read_text().splitlines()]
    assert len(baseline_all) == len(rows)
    baseline = [baseline_all[i] for i in selected]
    audit_path = REPO / 'tools/facts/joint-audit.py'
    spec = importlib.util.spec_from_file_location('unchanged_joint_audit', audit_path)
    audit = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(audit)
    old = audit.audit(cal, baseline)
    new = audit.audit(cal, predictions)
    link_counts = {name: {'linked': sum(p['entity'] is not None for p in stream),
                          'correct_entity': sum(p['entity'] == row[0] for row, p in zip(cal, stream, strict=True)),
                          'correct_entity_and_relation': sum(p['entity'] == row[0] and p['relation'] == row[1] for row, p in zip(cal, stream, strict=True))}
                   for name, stream in [('baseline', baseline), ('candidate', predictions)]}
    transitions = Counter()
    newly_wrong = []
    lost_correct = []
    for original_index, row, before, after in zip(selected, cal, baseline, predictions, strict=True):
        a, b = status(row, before), status(row, after)
        transitions[f'{a}_to_{b}'] += 1
        if a in ('right', 'refused') and b == 'wrong':
            newly_wrong.append({'train_index': original_index, 'gold': row[:3], 'question': row[3], 'baseline': before, 'candidate': after})
        if a == 'right' and b != 'right':
            lost_correct.append({'train_index': original_index, 'gold': row[:3], 'question': row[3], 'baseline': before, 'candidate': after})
    gain = new['joint_entity_relation_answer']['right'] - old['joint_entity_relation_answer']['right']
    passed = gain >= 10 and not newly_wrong
    db = sqlite3.connect(GRAPH.resolve().as_uri() + '?mode=ro', uri=True)
    db.executescript('PRAGMA query_only=ON; PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;')
    knowledge = native_knowledge(db, cal, predictions)
    db.close()
    report = {'scope': 'existing every-tenth TRAIN CAL; independent public first-shard graph', 'total': len(cal),
              'baseline': old, 'candidate': new, 'strict_joint_gain': gain,
              'link_counts': link_counts,
              'transitions': {f'{a}_to_{b}': transitions[f'{a}_to_{b}'] for a in ['right', 'refused', 'wrong'] for b in ['right', 'refused', 'wrong']},
              'newly_wrong': newly_wrong, 'lost_correct': lost_correct,
              'promotion_prerequisite_passed': passed, 'native_cal_knowledge_diagnostic': knowledge,
              'inference': inference, 'inputs': identities, 'joint_audit_tool': identity(audit_path),
              'seconds': time.monotonic() - started,
              'limitations': 'Development CAL, existing trained relation model, incomplete source shard, exact annotated IDs without redirect rewriting; no held-out or current-truth claim.'}
    (RUN / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    size = sum(path.stat().st_size for path in RUN.rglob('*') if path.is_file())
    assert size < 256 * 1024**2 and report['seconds'] < 600
    print(json.dumps({k: report[k] for k in ['total', 'strict_joint_gain', 'transitions', 'promotion_prerequisite_passed', 'native_cal_knowledge_diagnostic', 'seconds']}), flush=True)
    print(json.dumps({'baseline': old['joint_entity_relation_answer'], 'candidate': new['joint_entity_relation_answer'], 'newly_wrong': len(newly_wrong), 'lost_correct': len(lost_correct), 'run_bytes': size}), flush=True)


if __name__ == '__main__':
    main()
