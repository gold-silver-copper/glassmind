#!/usr/bin/env python3
"""TRAIN-only descriptive evidence; never called by runtime or DEV scorer."""
from pathlib import Path
from collections import Counter
import argparse
import hashlib
import json
import re

parser = argparse.ArgumentParser()
parser.add_argument('train', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
assert args.train.name == 'en_ewt-ud-train.conllu'
c, m, scalar = Counter(), Counter(), Counter()
n, words = 0, 0
for block in args.train.read_text().strip().split('\n\n'):
    n += 1
    lines = block.splitlines()
    text = next((line[9:] for line in lines if line.startswith('# text = ')), None)
    rows = [line.split('\t') for line in lines if not line.startswith('#')]
    forms = [r[1] for r in rows if r[0].isdigit()]
    words += len(forms)
    for row in rows:
        if row[0].isdigit() and any(a in row[1] for a in "'’"):
            c[row[1]] += 1
        if '-' in row[0] and any(a in row[1] for a in "'’"):
            m[row[1]] += 1
    if text:
        for match in re.finditer(r"[\w]+['’][\w]+", text):
            word = match.group()
            parts = re.findall(r"\w+|[^\w\s]", word)
            if any(forms[i:i+len(parts)] == parts for i in range(len(forms)-len(parts)+1)):
                scalar[word] += 1
out = {'split': 'official TRAIN only', 'source_sha256': hashlib.sha256(args.train.read_bytes()).hexdigest(),
       'sentences': n, 'integer_words': words, 'apostrophe_forms': c, 'apostrophe_mwt': m,
       'scalar_internal_apostrophe_sequences': scalar,
       'selection': 'Only productive apostrophe families retained; numeric/abbreviation families deferred. No word table in runtime.'}
args.output.write_text(json.dumps(out, ensure_ascii=False, indent=2)+'\n')
