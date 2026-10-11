#!/usr/bin/env python3
"""Independently replay a retained scalar witness with native Rust and public tests.

No expected answer is chosen: both saved outputs must match native execution,
and must differ. Optional --root contains only the permitted original source.
Artifacts and builds stay outside /tmp, using the existing CARGO_TARGET_DIR.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--witness', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--root', type=Path)
    args = parser.parse_args()
    evidence = json.loads(args.witness.read_text())
    w = evidence['witness']
    name = w['function']
    assert re.fullmatch(r'[A-Za-z_][A-Za-z_0-9]*', name)
    ty = w['argument'].split(':')[-1].strip()
    assert ty in ['bool'] + [f'{s}{b}' for s in 'iu' for b in (8, 16, 32, 64)]
    value = w['input']
    assert re.fullmatch(r'-?\d+|true|false', value)
    literal = value if ty == 'bool' else f'{value}{ty}'
    out = args.out.resolve()
    assert not out.is_relative_to('/tmp') and not out.exists()
    out.mkdir(parents=True)
    env = {k: v for k, v in os.environ.items() if not k.endswith('API_KEY')}
    target = Path(env['CARGO_TARGET_DIR'])
    env['TMPDIR'] = str(out / 'tmp')
    Path(env['TMPDIR']).mkdir()
    records = []

    def cargo(root, label):
        local = dict(env, CARGO_TARGET_DIR=str(target / 'ambiguity-replay' / label))
        command = ['systemd-run', '--user', '--scope', '-pMemoryMax=8G', '-pMemorySwapMax=0',
                   'setsid', '--wait', 'timeout', '--kill-after=10s', '600',
                   'cargo', 'test', '-j1', '--', '--test-threads', '4']
        with (out / f'{label}.log').open('w') as log:
            result = subprocess.run(command, cwd=root, env=local, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=620)
        records.append(dict(label=label, returncode=result.returncode, command=command))
        assert result.returncode == 0, label

    for side in ('first', 'second'):
        patch = evidence[f'{side}_patch']
        source = evidence['original_source']
        lines = source.splitlines(keepends=True)
        offset = sum(map(len, lines[:patch['line'] - 1])) + patch['column']
        assert source[offset:offset + len(patch['old'])] == patch['old']
        edited = source[:offset] + patch['new'] + source[offset + len(patch['old']):]
        assert edited == evidence[f'{side}_source']
        if args.root:
            original = args.root.resolve()
            relative = Path(patch['file'])
            assert not relative.is_absolute() and '..' not in relative.parts
            assert (original / relative).read_text() == source
            root = out / side
            shutil.copytree(original, root)
            (root / relative).write_text(edited)
            for p in original.rglob('*'):
                if p.is_file() and p.relative_to(original) != relative:
                    assert p.read_bytes() == (root / p.relative_to(original)).read_bytes()
            cargo(root, side)

    native = out / 'native'
    (native / 'src').mkdir(parents=True)
    (native / 'Cargo.toml').write_text('[package]\nname="ambiguity_native"\nversion="0.0.0"\nedition="2024"\n[workspace]\n')
    code = '\n'.join(f'mod {side} {{\n{evidence[f"{side}_source"]}\n}}' for side in ('first', 'second'))
    code += f'''\n#[test] fn retained_witness_matches_native_rust() {{
        let a = first::{name}({literal});
        let b = second::{name}({literal});
        assert_ne!(a, b);
        assert_eq!(format!("{{:?}}", a), {json.dumps(w['first_output'])});
        assert_eq!(format!("{{:?}}", b), {json.dumps(w['second_output'])});
    }}\n'''
    (native / 'src/lib.rs').write_text(code)
    cargo(native, 'native')
    (out / 'report.json').write_text(json.dumps(dict(witness=w, runs=records,
        witness_sha256=hashlib.sha256(args.witness.read_bytes()).hexdigest()), indent=2) + '\n')
    print(json.dumps(records))


if __name__ == '__main__':
    main()
