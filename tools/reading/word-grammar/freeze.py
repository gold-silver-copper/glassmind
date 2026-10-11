#!/usr/bin/env python3
"""Verify fixed input identities; freeze source, binary, and model BEFORE dev."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
mode, directory = sys.argv[1:]
run = Path(directory)
repo = Path(__file__).resolve().parents[3]
base = Path('/home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability')
dev = Path('/home/gsc/.cache/glassmind/grow-fast-20261010/reading-data/grammar.jsonl')
def record(path):
    data = path.read_bytes()
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
records = json.loads((repo/'tools/reading/word-grammar/verified-inputs.json').read_text())['inputs']
for path, expected in records.items():
    # Repository files relocate when coordinator replays; datasets remain mounted.
    original = Path(path)
    for marker in ['crates/', 'tools/reading/']:
        if marker in path:
            original = repo/path[path.index(marker):]
            break
    assert record(original) == expected, str(original)
if mode == 'verify':
    with (run/'grammar-sample.jsonl').open('xb') as f:
        f.write(b''.join(dev.read_bytes().splitlines(keepends=True)[:20]))
elif mode == 'freeze':
    target = Path(os.environ['CARGO_TARGET_DIR'])/'release/reading-word-probability'
    frozen = run/'reading-word-probability'
    assert not frozen.exists()
    shutil.copy2(target, frozen)
    paths = [repo/'crates/reading/src/word_probability.rs', repo/'crates/reading/src/bin/reading-word-probability.rs',
             repo/'crates/reading/src/probability.rs', repo/'crates/reading/Cargo.toml', repo/'Cargo.lock',
             frozen, run/'word3.bin', base/'byte6.bin', dev, run/'grammar-sample.jsonl']
    paths.extend(sorted((repo/'tools/reading/word-grammar').glob('*.py')))
    paths.extend([repo/'tools/reading/word-grammar/PLAN.md', repo/'tools/reading/word-grammar/reproduce.sh'])
    assert (run/'word3.bin').stat().st_size <= 150*1024*1024
    report = {'frozen_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'git_head': subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
              'artifacts': {str(p): record(p) for p in paths}}
    with (run/'frozen.json').open('x') as f: f.write(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
else:
    raise ValueError(mode)
