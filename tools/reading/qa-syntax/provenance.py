#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
import run as r,json,pathlib
# Includes copies of evaluated runtime source/binaries and complete raw/transition ledgers.
files=sorted(p for p in r.RUN.iterdir() if p.is_file())
files+=sorted(p for p in r.HERE.rglob('*') if p.is_file() and '__pycache__' not in p.parts and p.name!='provenance.json')
files += [r.ROOT/n for n in ['crates/reading/src/qa.rs','crates/reading/src/qa/syntax_v3.rs','crates/reading/src/bin/reading-qa-syntax.rs','crates/reading/Cargo.toml','Cargo.lock']]
r.write(r.REPORT/'provenance.json',dict(source=json.loads((r.q.SOURCE/'provenance.json').read_text()),syntax_training_source=json.loads((r.HERE.parent/'syntax/source-provenance.json').read_text()),baseline_manifest=json.loads((r.BASE/'manifest.json').read_text()),new_external_data_bytes=0,new_downloads=0,files=[dict(path=str(p),bytes=p.stat().st_size,sha256=r.q.digest(p.read_bytes())) for p in files]))
for x in json.loads((r.REPORT/'provenance.json').read_text())['files']:assert r.q.digest(pathlib.Path(x['path']).read_bytes())==x['sha256']
print('verified',len(files),'artifact hashes')
