#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Fixed authored public development fixtures; no benchmark inputs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

MANIFEST = '[package]\nname="isolation_probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n'
LOCK = '# Authored dependency-free lockfile\nversion = 4\n\n[[package]]\nname = "isolation_probe"\nversion = "0.0.0"\n'
CASES = {
    'pass': ('pub fn x() -> u8 { 7 }\n#[test] fn works() { assert_eq!(x(), 7); }\n', 'do the tests pass'),
    'fail': ('pub fn x() -> u8 { 7 }\n#[test] fn broken() { assert_eq!(x(), 8); }\n', 'do the tests pass'),
    'compiler': ('pub fn x() -> u8 { let n = 7; n += 1; n }\n#[test] fn works() { assert_eq!(x(), 8); }\n', 'fix the build'),
    'todo': ('pub fn x() -> u8 { todo!() }\n#[test] fn unfinished() { assert_eq!(x(), 8); }\n', 'repair x'),
}

def sha(data):
    return hashlib.sha256(data).hexdigest()

def snapshot(root):
    return {str(p.relative_to(root)): ('dir' if p.is_dir() else sha(p.read_bytes()))
            for p in sorted(root.rglob('*'))}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--sample', action='store_true')
    parser.add_argument('--require-pass', action='store_true')
    a = parser.parse_args()
    out = a.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    binary = a.binary.resolve()
    rows = []
    for owned in [False, True]:
        for lock in [False, True]:
            for name, (code, prompt) in CASES.items():
                if a.sample and (owned or lock or name not in ('pass', 'todo')):
                    continue
                tag = f'{name}-lock{int(lock)}-owned{int(owned)}'
                root = out / tag / 'package'
                (root / 'src').mkdir(parents=True)
                (root / 'target').mkdir()
                (root / '.kept').mkdir()
                (root / 'Cargo.toml').write_text(MANIFEST)
                (root / 'src/lib.rs').write_text(code)
                (root / 'target/sentinel').write_bytes(b'preserve build artifacts\x00')
                (root / '.kept/sentinel').write_bytes(b'preserve hidden bytes\x00')
                if lock:
                    (root / 'Cargo.lock').write_text(LOCK)
                before = snapshot(root)
                env = {k: v for k, v in os.environ.items()
                       if not k.endswith('API_KEY') and k not in
                       ('CARGO_TARGET_DIR', 'GLASSMIND_CARGO_TARGET', 'GLASSMIND_CHECK_CACHE', 'GLASSMIND_SCRATCH')}
                # Deliberately exercise ordinary HOME scratch fallback. The owning
                # process retains its prescribed CARGO_TARGET_DIR for builds.
                if owned:
                    env['GLASSMIND_CARGO_TARGET'] = str(Path(os.environ['CARGO_TARGET_DIR']) / 'isolation-owned')
                    env['GLASSMIND_CHECK_CACHE'] = str(out / 'owned-cache')
                run = subprocess.run([str(binary), 'do', str(root), prompt, '--explain'],
                                     env=env, capture_output=True, text=True, timeout=180)
                after = snapshot(root)
                delta = sorted(k for k in before.keys() | after.keys() if before.get(k) != after.get(k))
                reply = run.stdout
                expected = code.replace('let n =', 'let mut n =') if name == 'compiler' else code
                source_ok = (root / 'src/lib.rs').read_text() == expected
                diagnostic_ok = {
                    'pass': 'pass' in reply.lower() and 'failed' not in reply.lower(),
                    'fail': 'broken' in reply and 'failed' in reply.lower(),
                    'compiler': 'I made one change. No problems with the build.' in reply and 'E0384' in reply,
                    'todo': 'Remedy { check: Build, changes: [], fixed: [], left: [] }' in reply and 'Applied' not in reply,
                }[name]
                ok = run.returncode == 0 and source_ok and diagnostic_ok and delta == (['src/lib.rs'] if name == 'compiler' else [])
                row = dict(case=tag, passed=ok, exit=run.returncode, delta=delta,
                           source_ok=source_ok, diagnostic_ok=diagnostic_ok,
                           before=before, after=after, stdout=reply, stderr=run.stderr)
                rows.append(row)
                print(tag, 'PASS' if ok else 'FAIL', f'{len(delta)} changed entries', reply[-160:].replace('\n', ' '), flush=True)
                (out / 'results.json').write_text(json.dumps(dict(binary_sha256=sha(binary.read_bytes()),
                    fixture_sha256=sha((MANIFEST + LOCK + repr(CASES)).encode()),
                    passed=sum(r['passed'] for r in rows), total=len(rows), cases=rows), indent=2)+'\n')
    if a.require_pass and not all(r['passed'] for r in rows):
        raise SystemExit(1)

if __name__ == '__main__':
    main()
