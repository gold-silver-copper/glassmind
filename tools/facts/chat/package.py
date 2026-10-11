#!/usr/bin/env python3
"""Package local Wikidata assets with explicit provenance and SHA-256 identities.

Copies assets for an offline, relocatable bundle. Existing files must match;
choose a new output directory when replacing a bundle. No download or training.
The manifest detects accidental changes, not an untrusted publisher.
"""
import argparse
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--index', type=Path, required=True)
    parser.add_argument('--model', type=Path, required=True)
    parser.add_argument('--labels', type=Path)
    parser.add_argument('--source', required=True)
    parser.add_argument('--license', required=True)
    parser.add_argument('--scope', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    files = {'index.json': args.index.read_bytes(), 'relation.json': args.model.read_bytes()}
    manifest = dict(version=1, source=args.source, license=args.license, scope=args.scope,
                    index_sha256=hashlib.sha256(files['index.json']).hexdigest(),
                    relation_model_sha256=hashlib.sha256(files['relation.json']).hexdigest())
    if args.labels:
        files['labels.json'] = args.labels.read_bytes()
        if json.loads(files['labels.json'])['index_sha256'] != manifest['index_sha256']:
            parser.error('labels identify a different index')
    elif (args.output / 'labels.json').exists():
        parser.error('existing labels require --labels; choose a new bundle directory')
    files['manifest.json'] = (json.dumps(manifest, indent=2) + '\n').encode()
    for name, data in files.items():
        target = args.output / name
        if target.exists() and target.read_bytes() != data:
            parser.error(f'{target} already contains different data; choose a new directory')
    args.output.mkdir(parents=True, exist_ok=True)
    for name, data in files.items():
        target = args.output / name
        if not target.exists():
            with target.open('xb') as stream:
                stream.write(data)
    print(json.dumps(manifest))


if __name__ == '__main__':
    main()
