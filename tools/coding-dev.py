#!/usr/bin/env python3
"""Fixed training-only coding sample; verify repairs with independent cargo runs.

No benchmark hidden tests or generators are read. Run inside a capped scope.
"""
import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def copy_source(source, destination):
    shutil.copytree(source, destination, ignore=lambda _, names: [
        n for n in names if n.startswith('.') or n == 'target'
    ])


def cargo(root, check, env, log):
    args = ['check', '--lib', '--bins', '--tests', '--examples'] if check == 'build' else [
        'test', '--no-fail-fast']
    if not (root / 'src/lib.rs').exists() and '--lib' in args:
        args.remove('--lib')
    command = ['systemd-run', '--user', '--scope', '-pMemoryMax=8G',
               '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout',
               '--kill-after=5s', '600', 'cargo', *args, '-j', '1']
    if check == 'test':
        command += ['--', '--test-threads', '4']
    with log.open('w') as output:
        return subprocess.run(command, cwd=root, env=env, stdout=output,
                              stderr=subprocess.STDOUT).returncode == 0


def canonical(response):
    result = dict(response)
    if 'problems' in result:
        result['problems'] = sorted(result['problems'], key=lambda x: json.dumps(x, sort_keys=True))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--train', type=Path, required=True)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--baseline', action='store_true')
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    labels = {x['turn_id']: x for x in rows(args.train / 'labels.jsonl')}
    kinds = ('build_status', 'fix_build', 'test_status', 'fix_tests')
    counts = collections.Counter()
    sample = []
    for conversation in rows(args.train / 'inputs.jsonl'):
        if len(conversation['turns']) != 1:
            continue
        label = labels[conversation['turns'][0]['id']]
        kind = label['kind']
        if label['split'] == 'train' and kind in kinds and counts[kind] < 4:
            counts[kind] += 1
            sample.append(conversation)
    inputs = out / 'inputs.jsonl'
    inputs.write_text(''.join(json.dumps(c) + '\n' for c in sample))
    env = dict(os.environ)
    env.update(TMPDIR=str(out / 'tmp'), GLASSMIND_SCRATCH=str(out / 'scratch'),
               GLASSMIND_CARGO_TARGET=str(out / 'target'),
               GLASSMIND_CHECK_CACHE=str(out / 'checks'))
    Path(env['TMPDIR']).mkdir(exist_ok=True)
    command = [str(args.binary.resolve()), 'eval', '--sources', str(args.sources.resolve()),
               '--inputs', str(inputs), '--jobs', '1']
    if args.baseline:
        command += ['--disable', 'mind.accept_verified,coding.intent.fallback,rust.remedy.iterate_suggestions']
    prefix = 'baseline' if args.baseline else 'verified'
    outcomes = out / f'{prefix}-outcomes.jsonl'
    with outcomes.open('w') as stdout, (out / f'{prefix}-eval.log').open('w') as stderr:
        subprocess.run(command, env=env, stdout=stdout, stderr=stderr, check=True)
    totals = {k: collections.Counter() for k in kinds}
    details = []
    for index, outcome in enumerate(rows(outcomes)):
        label = labels[outcome['turn_id']]
        response = outcome['response']
        expected = label['expected']
        verdict = 'wrong'
        if response['type'] in ('refuse', 'ask'):
            verdict = 'refusal'
        elif expected['type'] == 'checked':
            verdict = 'right' if canonical(response) == canonical(expected) else 'wrong'
        elif response['type'] == 'remedy' and response['check'] == expected['check']:
            root = out / f'verify-{index}'
            if root.exists():
                shutil.rmtree(root)
            copy_source(args.sources / label['source'], root)
            grouped = collections.defaultdict(list)
            for change in response['changes']:
                assert change['source'] == label['source']
                relative = Path(change['file'])
                assert not relative.is_absolute() and '..' not in relative.parts
                grouped[relative].append(change)
            for relative, changes in grouped.items():
                path = root / relative
                text = path.read_text()
                lines = text.splitlines(keepends=True)
                for change in sorted(changes, key=lambda c: (c['line'], c['column']), reverse=True):
                    offset = sum(map(len, lines[:change['line'] - 1])) + change['column']
                    assert text[offset:offset + len(change['old'])] == change['old']
                    text = text[:offset] + change['new'] + text[offset + len(change['old']):]
                path.write_text(text)
            verify_env = dict(env, CARGO_TARGET_DIR=str(out / f'verify-target-{index}'))
            verdict = 'right' if cargo(root, expected['check'], verify_env,
                                       out / f'verify-{index}.log') else 'wrong'
            shutil.rmtree(root)
        totals[label['kind']][verdict] += 1
        details.append(dict(turn_id=outcome['turn_id'], kind=label['kind'], verdict=verdict))
    report = dict(totals=totals, turns=sum(counts.values()), details=details,
                  inputs_sha256=hashlib.sha256(inputs.read_bytes()).hexdigest())
    (out / f'{prefix}-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(totals, indent=2), flush=True)


if __name__ == '__main__':
    main()
