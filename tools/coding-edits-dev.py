#!/usr/bin/env python3
"""Independent, fixed rename examples; no dataset or hidden assets are read."""
import argparse
import json
import os
from pathlib import Path
import subprocess

# Source-binding checks include exact expected source alongside runtime checks.
CASES = [
    ('struct_cross_file', 'Packet', 'Message', True, {
        'src/main.rs': 'mod model; use crate::model::Packet; fn main() { let p: Packet = Packet { value: 7 }; println!("{} Packet", p.value()); }\n',
        'src/model.rs': '// Packet remains in this comment\npub struct Packet { pub value: u32 } impl Packet { pub fn value(&self) -> u32 { self.value } }\n',
    }),
    ('enum_paths', 'State', 'Phase', True, {
        'src/main.rs': 'mod model; use crate::model::State; fn read(s: State) -> u32 { match s { State::Ready(x) => x, State::Empty => 0 } } fn main() { let x = read(State::Ready(9)) + read(State::Empty); println!("{}", x); }\n',
        'src/model.rs': 'pub enum State { Ready(u32), Empty } impl State { pub fn empty() -> Self { Self::Empty } }\n',
    }),
    ('unicode_alias', 'État', 'Statut', True, {
        'src/main.rs': 'mod model; use crate::model::État as Local; fn main() { let café = Local(3); println!("{} État", café.0); }\n',
        'src/model.rs': 'pub struct État(pub u32);\n',
    }),
    ('shadow', 'Packet', 'Message', False, {'src/main.rs': 'struct Packet; fn other<Packet>(x: Packet) -> Packet { x } fn main() { let _ = Packet; println!("{}", other(4)); }\n'}),
    ('collision', 'Packet', 'Message', False, {'src/main.rs': 'struct Packet; struct Message; fn main() { let _ = Packet; let _ = Message; }\n'}),
    ('same_name', 'Packet', 'Message', False, {'src/main.rs': 'struct Packet; mod other { pub struct Packet; } fn main() { let _ = Packet; let _ = other::Packet; }\n'}),
    ('macro', 'Packet', 'Message', False, {'src/main.rs': 'struct Packet; fn main() { let _ = Packet; println!("{}", stringify!(Packet)); }\n'}),
    ('glob', 'Packet', 'Message', False, {'src/main.rs': 'mod model { pub struct Packet; } use model::*; fn main() { let _ = Packet; }\n'}),
    ('rollback', 'Packet', 'Message', False, {'src/main.rs': 'struct Packet; fn main() { let _ = Packet; missing(); }\n'}),
]


def cargo(root, *args):
    env = os.environ.copy()
    assert env.get('TMPDIR') and not env['TMPDIR'].startswith('/tmp')
    assert env.get('CARGO_TARGET_DIR')
    return subprocess.run(['systemd-run', '--user', '--scope', '-pMemoryMax=8G', '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout', '--kill-after=10s', '600', 'cargo', *args, '-j1'], cwd=root, env=env, text=True, capture_output=True, timeout=620)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--sample', action='store_true')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    results = []
    for name, old, new, supported, files in (CASES[:3] if a.sample else CASES):
        root = a.output / name
        root.mkdir(exist_ok=True)
        (root / 'Cargo.toml').write_text(f'[package]\nname = "rename_{name}"\nversion = "0.0.0"\nedition = "2021"\n[workspace]\n')
        for path, text in files.items():
            dest = root / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(text)
        before = cargo(root, 'run', '--quiet')
        done = subprocess.run([str(a.binary.resolve()), 'do', str(root.resolve()), f'Rename `{old}` to `{new}`'], capture_output=True, text=True, timeout=620)
        after_files = {path: (root / path).read_text() for path in files}
        changed = files != after_files
        after = cargo(root, 'run', '--quiet') if changed else before
        # Explicit expected source keeps strings/comments and aliases unchanged.
        expected = {}
        for path, text in files.items():
            expected[path] = text.replace(old, new).replace(f'// {new} remains', f'// {old} remains').replace(f'{{}} {new}"', f'{{}} {old}"')
        preserved = after_files == (expected if supported else files)
        behavior = before.returncode == after.returncode == 0 and before.stdout == after.stdout
        result = dict(case=name, supported=supported, changed=changed, exact_source=preserved, behavior=behavior, before_exit=before.returncode, after_exit=after.returncode, reply=done.stdout, stderr=done.stderr)
        result['pass'] = preserved and (behavior if supported else not changed)
        results.append(result)
        print(json.dumps(result), flush=True)
    (a.output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')

if __name__ == '__main__':
    main()
