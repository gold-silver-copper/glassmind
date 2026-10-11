#!/usr/bin/env python3
"""Extract only authorized training members; hash frozen dev and make fixed samples."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

p = argparse.ArgumentParser()
p.add_argument('--cache', type=Path, required=True)
p.add_argument('--archive', type=Path, default=Path('/home/gsc/code/sememe/data/babylm/train_10M.zip'))
p.add_argument('--readme', type=Path, default=Path('/home/gsc/code/sememe/data/babylm/README.md'))
p.add_argument('--dev', type=Path, default=Path('/home/gsc/.cache/glassmind/grow-fast-20261010/reading-data'))
a = p.parse_args()
a.cache.mkdir(parents=True, exist_ok=True)

def record(data):
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}

metadata = {'archive': record(a.archive.read_bytes()), 'readme': record(a.readme.read_bytes()), 'members': {}, 'dev': {}}
names = ['bnc_spoken', 'childes', 'gutenberg', 'open_subtitles', 'simple_wiki', 'switchboard']
with zipfile.ZipFile(a.archive) as archive:
    for name in names:
        member = f'train_10M/{name}.train'
        data = archive.read(member)
        target = a.cache / (name + '.train')
        target.write_bytes(data)
        metadata['members'][member] = record(data)
for name, count in [('grammar', 20), ('reading', 12)]:
    data = (a.dev / (name + '.jsonl')).read_bytes()
    metadata['dev'][name] = record(data)
    rows = data.splitlines(keepends=True)
    metadata['dev'][name]['rows'] = len(rows)
    (a.cache / (name + '-sample.jsonl')).write_bytes(b''.join(rows[:count]))
(a.cache / 'provenance.json').write_text(json.dumps(metadata, indent=2) + '\n')
print(json.dumps(metadata, indent=2))
