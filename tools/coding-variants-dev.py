#!/usr/bin/env python3
"""Fixed independent tiny-crate dev checks. No benchmark assets are read."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

# Each pair is literal before/after source. Refusal cases have identical pairs.
CASES = [
    ('constructors', 'Signal::Ready', 'Active', {
        'src/main.rs': (
            'enum Signal { Ready(u32), Empty, Named { n: u32 } } // Ready stays\nfn read(s: Signal) -> u32 { match s { Signal::Ready(n) => n, Signal::Empty => 0, Signal::Named { n } => n } } fn main() { let n = read(Signal::Ready(7)) + read(Signal::Empty) + read(Signal::Named { n: 2 }); println!("{} Ready", n); }\n',
            'enum Signal { Active(u32), Empty, Named { n: u32 } } // Ready stays\nfn read(s: Signal) -> u32 { match s { Signal::Active(n) => n, Signal::Empty => 0, Signal::Named { n } => n } } fn main() { let n = read(Signal::Active(7)) + read(Signal::Empty) + read(Signal::Named { n: 2 }); println!("{} Ready", n); }\n')}, True),
    ('nested_alias', 'State::Ready', 'Active', {
        'src/main.rs': ('mod outer; use crate::outer::model::State as Status; use crate::outer::model::State::Ready as Go; fn main() { let n = match Go(8) { Status::Ready(n) => n }; println!("{}", n); }\n', 'mod outer; use crate::outer::model::State as Status; use crate::outer::model::State::Active as Go; fn main() { let n = match Go(8) { Status::Active(n) => n }; println!("{}", n); }\n'),
        'src/outer/mod.rs': ('pub mod model;\n', 'pub mod model;\n'),
        'src/outer/model.rs': ('pub enum State { Ready(u32) }\n', 'pub enum State { Active(u32) }\n')}, True),
    ('library_unicode', 'État::Prêt', 'Actif', {
        'src/lib.rs': ('pub enum État<T> { Prêt { value: T } } impl<T> État<T> { pub fn make(value: T) -> Self { Self::Prêt { value } } pub fn value(self) -> T { match self { Self::Prêt { value } => value } } } pub fn exercise() -> u32 { let café = État::make(6); café.value() }\n', 'pub enum État<T> { Actif { value: T } } impl<T> État<T> { pub fn make(value: T) -> Self { Self::Actif { value } } pub fn value(self) -> T { match self { Self::Actif { value } => value } } } pub fn exercise() -> u32 { let café = État::make(6); café.value() }\n')}, True),
    ('unit_import', 'State::Ready', 'Active', {
        'src/main.rs': ('enum State { Ready, Empty } use self::State::{Ready, Empty}; fn read(s: State) -> u32 { match s { Ready => 3, Empty => 0 } } fn main() { let n = read(Ready); println!("{}", n); }\n', 'enum State { Active, Empty } use self::State::{Active, Empty}; fn read(s: State) -> u32 { match s { Active => 3, Empty => 0 } } fn main() { let n = read(Active); println!("{}", n); }\n')}, True),
]
NEGATIVES = [
    ('ambiguous', 'enum State { Ready } enum Other { Ready } fn main() { let _ = State::Ready; let _ = Other::Ready; }'),
    ('generic_shadow', 'enum State { Ready } trait Trait { const Ready: u32; } fn f<State: Trait>() -> u32 { State::Ready } fn main() { let _ = State::Ready; }'),
    ('type_alias', 'enum State { Ready } type Alias = State; fn main() { let _ = Alias::Ready; }'),
    ('module_collision', 'enum State { Ready } mod other { pub mod State { pub const Ready: u32 = 4; } pub fn f() -> u32 { State::Ready } } fn main() { let _ = State::Ready; let _ = other::f(); }'),
    ('raw', 'enum State { Ready } fn main() { let _ = State::r#Ready; }'),
    ('collision', 'enum State { Ready, Active } fn main() { let _ = State::Ready; }'),
    ('macro', 'enum State { Ready } fn main() { let _ = State::Ready; println!("{}", stringify!(Ready)); }'),
    ('glob', 'enum State { Ready } use State::*; fn main() { let _ = Ready; }'),
    ('local_shadow', 'enum State { Ready } fn f(Ready: u32) -> u32 { Ready } fn main() { let _ = State::Ready; let _ = f(4); }'),
    ('rollback', 'enum State { Ready } fn main() { let _ = State::Ready; missing(); }'),
]
for name, code in NEGATIVES:
    CASES.append((name, 'State::Ready', 'Active', {'src/main.rs': (code + '\n', code + '\n')}, False))


def cargo(root, *args):
    env = os.environ.copy()
    assert env.get('TMPDIR') and not env['TMPDIR'].startswith('/tmp')
    assert env.get('CARGO_TARGET_DIR')
    for key in list(env):
        if key.endswith('API_KEY'):
            del env[key]
    return subprocess.run(['systemd-run', '--user', '--scope', '-pMemoryMax=8G', '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout', '--kill-after=10s', '600', 'cargo', *args, '-j1'], cwd=root, env=env, capture_output=True, text=True, timeout=620)


def runtime(root, library):
    if not library:
        return cargo(root, 'run', '--quiet')
    # External harness is outside indexed source and only calls public API.
    runner = root.parent / (root.name + '_runner')
    (runner / 'src').mkdir(parents=True, exist_ok=True)
    (runner / 'Cargo.toml').write_text('[package]\nname="variant_runner"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[dependencies]\nvariant_case={path=' + json.dumps(str(root.resolve())) + '}\n')
    (runner / 'src/main.rs').write_text('fn main() { println!("{}", variant_case::exercise()); }\n')
    return cargo(runner, 'run', '--quiet')


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--sample', action='store_true')
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    rows = []
    for name, old, new, sources, supported in (CASES[:3] if a.sample else CASES):
        root = a.output / name
        root.mkdir(exist_ok=True)
        (root / 'Cargo.toml').write_text('[package]\nname="variant_case"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
        for path, (before, _) in sources.items():
            dest = root / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(before)
        before = runtime(root, 'src/lib.rs' in sources)
        done = subprocess.run([str(a.binary.resolve()), 'do', str(root.resolve()), f'Rename `{old}` to `{new}`'], capture_output=True, text=True, timeout=620)
        actual = {path: (root / path).read_text() for path in sources}
        changed = any(actual[path] != pair[0] for path, pair in sources.items())
        after = runtime(root, 'src/lib.rs' in sources) if changed else before
        exact = all(actual[path] == pair[1] for path, pair in sources.items())
        behavior = before.returncode == after.returncode == 0 and before.stdout == after.stdout
        row = dict(case=name, supported=supported, changed=changed, exact_source=exact, behavior=behavior, before_exit=before.returncode, after_exit=after.returncode, before_stdout=before.stdout, after_stdout=after.stdout, reply=done.stdout, stderr=done.stderr)
        row['pass'] = exact and (behavior and changed if supported else not changed)
        rows.append(row)
        print(json.dumps(row), flush=True)
    result = dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), cases=rows, passed=sum(r['pass'] for r in rows), total=len(rows))
    (a.output / 'results.json').write_text(json.dumps(result, indent=2) + '\n')

if __name__ == '__main__':
    main()
