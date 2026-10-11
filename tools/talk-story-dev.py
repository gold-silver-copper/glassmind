#!/usr/bin/env python3
"""Development-only talk integration checks; no labels enter the CLI.

bAbI uses the unchanged training-only partition and normalization in stories/babi.py.
Mixed is a fixed same-author product check, not an independent benchmark.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile


def run(binary, lines, session=None, model=None):
    command = [str(binary.resolve()), 'talk', '--json']
    if session:
        command += ['--session', str(session)]
    if model:
        command += ['--model', str(model)]
    result = subprocess.run(command, input='\n'.join(lines) + '\n', text=True,
                            capture_output=True, check=True, timeout=550)
    replies = [json.loads(line) for line in result.stdout.splitlines()]
    if len(replies) != len(lines):
        raise ValueError('one response required per turn')
    return replies


def mixed(args):
    results = []
    def check(rows, session):
        replies = run(args.binary, [r[0] for r in rows], session, args.model)
        for (text, expected), reply in zip(rows, replies, strict=True):
            correct = all(reply.get(k) == v for k, v in expected.items() if k != 'contains')
            correct &= expected.get('contains', '') in reply.get('text', '')
            status = 'right' if correct else 'refused' if reply.get('kind') in {'refusal', 'intent'} else 'wrong'
            results.append(dict(utterance=text, expected=expected, actual=reply, outcome=status))
    fact = {'kind': 'story_fact'}
    def answer(value):
        return {'kind': 'story_answer', 'answer': value}
    with tempfile.TemporaryDirectory(dir=args.output.parent, prefix='talk-story-') as tmp:
        session = Path(tmp) / 'session.json'
        check([
            ('My name is Mira', {'kind': 'remembered'}),
            ('Mary went to the kitchen.', fact),
            ('Mary picked up the apple.', fact),
            ('Calculate 2 + 3 * 4', {'kind': 'calculated', 'value': '14'}),
            ('Hello', {'kind': 'social'}),
            ('Mary moved to the garden.', fact),
            ('Where is the apple?', answer('garden')),
            ('What did I just say?', {'kind': 'recalled', 'source_turn': 7, 'contains': 'Where is the apple?'}),
            ('What is my name?', {'kind': 'recalled', 'source_turn': 1}),
            ('Mary gave the apple to John.', fact),
            ('Who has the apple?', answer('john')),
            ('John dropped the apple.', fact),
            ('How many objects is John carrying?', answer('none')),
            ('Actually, Mary is in the bedroom.', fact),
            ('Where is Mary?', answer('bedroom')),
            ('Mary is either in the kitchen or the garden.', fact),
            ('Where is Mary?', {'kind': 'refusal'}),
            ('Is Mary in the kitchen?', answer('maybe')),
            ('Mary is in the office.', fact),
            ('My favorite color is green', {'kind': 'remembered'}),
        ], session)
        check([
            ('Where is Mary?', answer('office')),
            ('Where is the apple?', answer('garden')),
            ('What is my favorite color?', {'kind': 'recalled', 'source_turn': 20}),
            ('Mary teleported to the attic.', {'kind': 'refusal'}),
            ('Where is Mary?', {'kind': 'refusal'}),
            ('Calculate 7 * 8', {'kind': 'calculated', 'value': '56'}),
            ('What is my name?', {'kind': 'recalled', 'source_turn': 1}),
        ], session)
        check([
            ('Where is the apple?', {'kind': 'refusal'}),
            ('/reset', {'kind': 'reset'}),
            ('Where is Mary?', {'kind': 'refusal'}),
            ('What is my name?', {'kind': 'missing'}),
            ('The kitchen is north of the office.', fact),
            ('What is north of the office?', answer('kitchen')),
            ('What did I just say?', {'kind': 'recalled', 'source_turn': 4}),
            ('Actually, the kitchen is south of the office.', {'kind': 'refusal'}),
            ('What is north of the office?', {'kind': 'refusal'}),
            ('/reset', {'kind': 'reset'}),
            ('Lily is a swan.', fact),
            ('Lily is white.', fact),
            ('Ruby is a swan.', fact),
            ('What color is Ruby?', {'kind': 'story_prediction', 'answer': 'white'}),
            ('/reset', {'kind': 'reset'}),
            ('Bill is hungry.', fact),
            ('Where will Bill go?', {'kind': 'story_prediction', 'answer': 'kitchen'}),
        ], session)
        # Legacy root-level memory schema, with no story field.
        session.write_text(json.dumps(dict(turn=1, name=dict(value='Ada', stated_on_turn=1),
            preferences={}, last_slot='Name', last_user_utterance='My name is Ada', last_correction=None)))
        check([('What is my name?', {'kind': 'recalled', 'source_turn': 1, 'contains': 'Ada'}),
               ('Ada went to the hall.', fact), ('Where is Ada?', answer('hall'))], session)
    return dict(dataset='talk-stories-mixed-dev-v1', independent_benchmark=False,
                total=len(results), totals=dict(Counter(r['outcome'] for r in results)), outcomes=results)


def babi(args):
    spec = importlib.util.spec_from_file_location('babi', Path(__file__).parent / 'stories/babi.py')
    source = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source)
    rows = list(source.items(args.data, 'dev', args.sample))
    lines, indices = [], []
    for row in rows:
        lines += ['/reset', *row['story'], row['question']]
        indices.append(len(lines) - 1)
    replies = run(args.binary, lines)
    totals, tasks, failures = Counter(), defaultdict(Counter), []
    for row, index in zip(rows, indices, strict=True):
        reply = replies[index]
        answer = reply.get('answer')
        status = 'refusal' if answer is None else 'correct' if source.normalize(answer, row['task']) == source.normalize(row['expected'], row['task']) else 'wrong'
        totals[status] += 1
        tasks[row['task']][status] += 1
        if status != 'correct':
            failures.append(dict(id=row['id'], expected=row['expected'], actual=reply, outcome=status))
    return dict(dataset='bAbI-v1.2-training-partition-through-talk', split='dev', seed=source.SEED,
                sample_per_task=args.sample, total=len(rows), totals=dict(totals),
                item_ids_sha256=source.digest('\n'.join(r['id'] for r in rows).encode()),
                per_task=dict(tasks), failures=failures)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--mode', choices=['mixed', 'babi'], default='mixed')
    parser.add_argument('--model', type=Path)
    parser.add_argument('--data', type=Path)
    parser.add_argument('--sample', type=int, default=20, help='per task; 0 = full dev')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    report = mixed(args) if args.mode == 'mixed' else babi(args)
    report['binary_sha256'] = hashlib.sha256(args.binary.read_bytes()).hexdigest()
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in {'outcomes', 'failures', 'per_task'}}))


if __name__ == '__main__':
    main()
