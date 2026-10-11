#!/usr/bin/env python3
"""Post-evaluation diagnostic only: never train, tune, score, or filter dev rows."""
import argparse
import json
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--cache', type=Path, required=True)
p.add_argument('--dev', type=Path, required=True)
a = p.parse_args()
texts = [json.loads(line)['text'].encode('utf-8') for line in a.dev.read_text().splitlines()]
wanted, found = set(texts), set()
for name in ['bnc_spoken', 'childes', 'gutenberg', 'open_subtitles', 'simple_wiki', 'switchboard']:
    with (a.cache / (name + '.train')).open('rb') as corpus:
        for line in corpus:
            if line.endswith(b'\n'):
                line = line[:-1]
                if line.endswith(b'\r'):
                    line = line[:-1]
            if line in wanted:
                found.add(line)
print(json.dumps({'policy': 'post-evaluation exact byte equality against entire training lines; diagnostic only, no fitting or filtering',
                  'documents': len(texts), 'exact_training_line_matches': sum(t in found for t in texts)}, indent=2))
