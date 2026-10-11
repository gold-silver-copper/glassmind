#!/usr/bin/env python3
"""Freeze this bounded batch after validation; no secrets or unrelated logs."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

workspace = Path.cwd()
run = workspace.parent / 'unitcompare-run'
source = workspace.parent / 'unitcompare-source'
frozen = run / 'frozen-candidate'
# Ensure the preimplementation contract/source snapshot was not silently changed.
original = json.loads((frozen / 'original-hashes.json').read_text())
for name, entry in original.items():
    data = (frozen / name).read_bytes()
    assert len(data) == entry['bytes'] and hashlib.sha256(data).hexdigest() == entry['sha256'], name
for name in ['PLAN.md', 'authored.json', 'freeze.py']:
    assert (workspace / 'tools/quantities/unit-comparison' / name).read_bytes() == (frozen / name).read_bytes(), name
for owned in ['crates/quantities', 'tools/quantities/unit-comparison']:
    shutil.copytree(workspace / owned, frozen / 'final-source' / owned, dirs_exist_ok=True)
for name in ['Cargo.toml', 'Cargo.lock']:
    shutil.copy2(workspace / name, frozen / 'final-source' / name)
    shutil.copy2(workspace / name, frozen / 'original-source' / name)
shutil.copy2(run / 'target/release/glassmind-quantities', frozen / 'final-native')
shutil.copy2(run / 'target/release/examples/compare_typed', frozen / 'compare-typed')
shutil.copytree(run / 'review', frozen / 'review', dirs_exist_ok=True)
shutil.copytree(run / 'failures', frozen / 'failures', dirs_exist_ok=True)
(frozen / 'checks').mkdir(exist_ok=True)
for name in ['fmt-initial.log', 'tests-initial.log', 'fmt-final.log', 'clippy-initial.log',
             'format-final.log', 'clippy-final.log', 'release-final.log', 'tests-final.log',
             'fmt-verified.log', 'review.log']:
    shutil.copy2(run / name, frozen / 'checks' / name)
(frozen / 'compatibility-inputs').mkdir(exist_ok=True)
for name in ['manifest.json', 'sample-questions.jsonl', 'sample-baseline.jsonl',
             'full-questions.jsonl', 'full-baseline.jsonl', 'synthetic-questions.jsonl', 'synthetic-baseline.jsonl']:
    shutil.copy2(source / name, frozen / 'compatibility-inputs' / name)
owned = ['crates/quantities', 'tools/quantities/unit-comparison']
patch = subprocess.check_output(['git', 'diff', '--binary', '--', *owned])
new = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '--', *owned], text=True).splitlines()
for name in new:
    p = subprocess.run(['git', 'diff', '--no-index', '--binary', '--', '/dev/null', name], capture_output=True)
    assert p.returncode in [0,1]
    patch += p.stdout
(frozen / 'candidate.patch').write_bytes(patch)
(frozen / 'git-status.txt').write_bytes(subprocess.check_output(['git', 'status', '--short']))
(frozen / 'commands.md').write_text((workspace / 'tools/quantities/unit-comparison/HANDOFF.md').read_text())
files = {str(p.relative_to(frozen)): dict(bytes=p.stat().st_size, sha256=hashlib.sha256(p.read_bytes()).hexdigest())
         for p in sorted(frozen.rglob('*')) if p.is_file() and p.name != 'final-hashes.json'}
(frozen / 'final-hashes.json').write_text(json.dumps(files, indent=2)+'\n')
for name, entry in files.items():
    p = frozen / name
    assert hashlib.sha256(p.read_bytes()).hexdigest() == entry['sha256']
print(json.dumps(dict(files=len(files), bytes=sum(v['bytes'] for v in files.values()),
                     manifest_sha256=hashlib.sha256((frozen/'final-hashes.json').read_bytes()).hexdigest()),indent=2))
