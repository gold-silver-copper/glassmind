#!/usr/bin/env python3
"""Fetch only pinned official MathQA train/dev files; no dataset loader/archive."""
import argparse
import hashlib
import json
from pathlib import Path
import urllib.request

ap = argparse.ArgumentParser()
ap.add_argument('--data', type=Path, required=True)
args = ap.parse_args()
manifest = json.loads((Path(__file__).parent / 'reports/source.json').read_text())
args.data.mkdir(parents=True, exist_ok=True)
for entry in manifest['files']:
    assert entry['split'] in ('train', 'validation')
    path = args.data / (entry['split'] + '.parquet')
    if path.exists():
        content = path.read_bytes()
    else:
        with urllib.request.urlopen(entry['url'], timeout=60) as response:
            content = response.read(entry['bytes'] + 1)
    if len(content) != entry['bytes'] or hashlib.sha256(content).hexdigest() != entry['sha256']:
        raise ValueError(f"size or hash mismatch: {entry['split']}")
    if not path.exists():
        path.write_bytes(content)
    print(entry['split'], len(content), entry['sha256'])
