#!/usr/bin/env python3
"""Training-only repair probe. Labels supply only kind/split, never patches.

This reports visible cargo verification, not benchmark correctness. Run under a
memory-capped scope. All copies and builds stay under the supplied output and
existing CARGO_TARGET_DIR; no hidden assets are accessed.
"""
import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def rows(path):
    with path.open() as stream:
        return [json.loads(line) for line in stream if line.strip()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--train', type=Path, required=True)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--sample', action='store_true')
    parser.add_argument('--check-cache', type=Path, help='Reuse content-fingerprinted cargo observations')
    args = parser.parse_args()
    labels = {r['turn_id']: (r['kind'], r['split'])
              for r in rows(args.train / 'labels.jsonl')}
    groups = collections.defaultdict(list)
    seen = set()
    for conversation in rows(args.train / 'inputs.jsonl'):
        if len(conversation['turns']) != 1:
            continue
        kind, split = labels[conversation['turns'][0]['id']]
        source = conversation['sources'][0]
        if (split == 'train' and kind in ('fix_tests', 'rb_fix_test')
                and source not in seen and (args.sources / source).is_dir()):
            groups[kind].append(conversation)
            seen.add(source)
    # Freeze file-order selection before inspecting outcomes.
    sample = groups['fix_tests'][:4] + groups['rb_fix_test'][::7 if args.sample else 1]
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    sources = out / 'sources'
    sources.mkdir(exist_ok=True)
    env = {k: v for k, v in os.environ.items() if not k.endswith('API_KEY')}
    target = Path(env['CARGO_TARGET_DIR'])
    env.update(TMPDIR=str(out / 'tmp'), GLASSMIND_SCRATCH=str(out / 'scratch'),
               GLASSMIND_CARGO_TARGET=str(target / 'repair-dev'),
               GLASSMIND_CHECK_CACHE=str((args.check_cache or out / 'checks').resolve()))
    Path(env['TMPDIR']).mkdir(exist_ok=True)
    inputs = out / 'inputs.jsonl'
    inputs.write_text(''.join(json.dumps(c) + '\n' for c in sample))
    details = []
    for index, conversation in enumerate(sample):
        source = conversation['sources'][0]
        root = sources / source
        if root.exists():
            shutil.rmtree(root)
        shutil.copytree(args.sources / source, root, ignore=lambda _, names: [
            n for n in names if n.startswith('.') or n == 'target' or 'hidden' in n.lower()
        ])
        one = out / 'one.jsonl'
        one.write_text(json.dumps(conversation) + '\n')
        with (out / f'{index}-eval.log').open('w') as log:
            result = subprocess.run([str(args.binary.resolve()), 'eval', '--sources', str(sources),
                                     '--inputs', str(one), '--jobs', '1'], env=env,
                                    text=True, stdout=subprocess.PIPE, stderr=log, check=True)
        outcome = json.loads(result.stdout)
        response = outcome['response']
        (out / f'{index}-outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
        verdict = 'refused'
        changes = response.get('changes', [])
        if response['type'] == 'remedy' and changes and not response.get('left'):
            for change in sorted(changes, key=lambda c: (c['file'], c['line'], c['column']), reverse=True):
                relative = Path(change['file'])
                assert change['source'] == source and not relative.is_absolute() and '..' not in relative.parts
                path = root / relative
                text = path.read_text()
                offset = sum(map(len, text.splitlines(keepends=True)[:change['line'] - 1])) + change['column']
                assert text[offset:offset + len(change['old'])] == change['old']
                path.write_text(text[:offset] + change['new'] + text[offset + len(change['old']):])
            for path in root.rglob('*'):
                if path.is_file():
                    os.utime(path, (time.time() + 86400, time.time() + 86400))
            verify_env = dict(env, CARGO_TARGET_DIR=str(target / 'repair-verify' / source))
            with (out / f'{index}-verify.log').open('w') as log:
                verified = subprocess.run(['systemd-run', '--user', '--scope', '-pMemoryMax=8G',
                    '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout', '--kill-after=10s', '600',
                    'cargo', 'test', '-j1', '--no-fail-fast', '--', '--test-threads', '4'],
                    cwd=root, env=verify_env, stdout=log, stderr=subprocess.STDOUT)
            verdict = 'verified' if verified.returncode == 0 else 'failed_verification'
        details.append(dict(source=source, turn_id=outcome['turn_id'], verdict=verdict, changes=changes))
        print(source, verdict, flush=True)
        shutil.rmtree(root)
        report = dict(totals=dict(collections.Counter(d['verdict'] for d in details)),
                      details=details, inputs_sha256=hashlib.sha256(inputs.read_bytes()).hexdigest())
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
