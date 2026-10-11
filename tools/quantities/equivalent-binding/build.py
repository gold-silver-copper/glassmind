import os
from pathlib import Path
import shutil,json,hashlib,difflib
ROOT=Path.cwd();S=ROOT.parent/'groundingaudit-source';R=Path(os.environ.get('EQUIV_RUN',str(ROOT.parent/'equivbinding-run/experiment')));O=ROOT/'tools/quantities/equivalent-binding';C=R/'helper'
(C/'src').mkdir(parents=True,exist_ok=True)
for name in ['proposal.rs','rational.rs']:shutil.copy2(S/'rejected-source/crates/quantities/src'/name,C/'src'/name)
with (C/'src/proposal.rs').open('a') as f:f.write('\ninclude!("instrument.rs");\ninclude!("checks.rs");\n')
for name in ['instrument.rs','main.rs','checks.rs']:shutil.copy2(O/name,C/'src'/name)
(C/'Cargo.toml').write_text('''[package]
name = "equivalent-binding-helper"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
sha2 = "0.10"
''')
shutil.copy2(ROOT/'Cargo.lock',C/'Cargo.lock')
original=(S/'rejected-source/crates/quantities/src/proposal.rs').read_text();copied=(C/'src/proposal.rs').read_text()
(R/'instrumentation.diff').write_text(''.join(difflib.unified_diff(original.splitlines(True),copied.splitlines(True),fromfile='original/proposal.rs',tofile='helper/proposal.rs')))
