#!/usr/bin/env python3
"""Train counts for glassmind_models::IntentModel, excluding dev source groups."""
import argparse
import collections
import json
from pathlib import Path
import re


def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--train', type=Path, required=True)
    parser.add_argument('--exclude-inputs', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    excluded = {s for c in rows(args.exclude_inputs) for s in c['sources']}
    labels = {l['turn_id']: l for l in rows(args.train / 'labels.jsonl') if l['split'] == 'train'}
    templates = collections.Counter()
    seen = collections.Counter()
    counts = collections.Counter()
    aliases = {'rb_fix_error': 'fix_build', 'rb_fix_test': 'fix_tests'}
    for conversation in rows(args.train / 'inputs.jsonl'):
        if excluded.intersection(conversation['sources']):
            continue
        for turn in conversation['turns']:
            label = labels.get(turn['id'])
            if label is None:
                continue
            kind = aliases.get(label['kind'], label['kind'])
            if kind not in ('build_status', 'test_status', 'fix_build', 'fix_tests'):
                kind = 'other'
            words = set(re.findall('[a-z]+', turn['text'].lower(), re.ASCII))
            templates[kind] += 1
            seen.update(words)
            counts.update((kind, w) for w in words)
    kept = {w for w, n in seen.items() if n >= 2}
    lines = [f'examples\t{sum(templates.values())}', 'sure\t0.98', 'offer\t0.98']
    lines += [f'template\t{k}\t{v}' for k, v in sorted(templates.items())]
    lines += [f'seen\t{w}\t{seen[w]}' for w in sorted(kept)]
    lines += [f'word\t{k}\t{w}\t{n}' for (k, w), n in sorted(counts.items()) if w in kept]
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text('\n'.join(lines) + '\n')
    print(json.dumps(dict(examples=sum(templates.values()), classes=templates,
                          excluded_sources=len(excluded), words=len(kept),
                          bytes=args.out.stat().st_size), indent=2))


if __name__ == '__main__':
    main()
