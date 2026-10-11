#!/usr/bin/env python3
"""Fixed independent package fixtures and permitted workspace-only prompt replay."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

MANIFEST = '[package]\nname="parcel-box"\nversion="0.0.0"\nedition="2021"\n[lib]\nname="parcel_api"\n[workspace]\n'
CASES = [
    ('unicode_reexport', 'État', 'Statut', True, {
        'src/lib.rs': ('//! État is documentation.\nmod model { /// État stays.\npub struct État(pub u32); } pub use model::État; pub mod api { pub use crate::État; } use model::État as Local; pub fn value() -> u32 { let café = Local(5); café.0 }\n', '//! État is documentation.\nmod model { /// État stays.\npub struct Statut(pub u32); } pub use model::Statut; pub mod api { pub use crate::Statut; } use model::Statut as Local; pub fn value() -> u32 { let café = Local(5); café.0 }\n'),
        'src/main.rs': ('use parcel_api::api::État as Local; fn main() { let café = Local(7); let n = café.0 + parcel_api::value(); println!("{} État", n); } // État stays\n', 'use parcel_api::api::Statut as Local; fn main() { let café = Local(7); let n = café.0 + parcel_api::value(); println!("{} État", n); } // État stays\n'),
    }),
    ('variant_reexport', 'State::Ready', 'Active', True, {
        'src/lib.rs': ('mod model { pub enum State { Ready(u32) } } pub use model::State; pub mod api { pub use crate::State; pub use crate::State::Ready as Go; }\n', 'mod model { pub enum State { Active(u32) } } pub use model::State; pub mod api { pub use crate::State; pub use crate::State::Active as Go; }\n'),
        'src/main.rs': ('use parcel_api::api::{State, Go}; fn main() { let n = match Go(9) { State::Ready(n) => n }; println!("{} Ready", n); }\n', 'use parcel_api::api::{State, Go}; fn main() { let n = match Go(9) { State::Active(n) => n }; println!("{} Ready", n); }\n'),
    }),
    ('builtin_derives', 'Parcel', 'Packet', True, {
        'src/lib.rs': ('/// Parcel docs\n#[derive(Clone, Copy, PartialEq, Eq, Default)] pub struct Parcel { pub n: u32 }\n', '/// Parcel docs\n#[derive(Clone, Copy, PartialEq, Eq, Default)] pub struct Packet { pub n: u32 }\n'),
        'src/main.rs': ('use parcel_api::Parcel; fn main() { let p = Parcel::default(); let n = p.n; println!("{}", n); }\n', 'use parcel_api::Packet; fn main() { let p = Packet::default(); let n = p.n; println!("{}", n); }\n'),
    }),
]
BASE_LIB = 'pub struct Parcel(pub u32);\n'
BASE_MAIN = 'use parcel_api::Parcel; fn main() { let p = Parcel(3); let n = p.0; println!("{}", n); }\n'
for name, lib, main, extra in [
    ('debug', '#[derive(Debug)] '+BASE_LIB, BASE_MAIN, {}),
    ('custom_derive', '#[derive(Custom)] '+BASE_LIB, BASE_MAIN, {}),
    ('cfg', '#[cfg(unix)] '+BASE_LIB, BASE_MAIN, {}),
    ('macro', BASE_LIB, BASE_MAIN.replace('let p', 'unknown!(); let p'), {}),
    ('cycle', BASE_LIB+'pub use a::Alias; mod a { pub use crate::Alias; }\n', BASE_MAIN, {}),
    ('shadow', BASE_LIB, 'use parcel_api as api; fn f<api>() {} fn main() { let _ = api::Parcel(3); }\n', {}),
    ('external', BASE_LIB+'use missing_crate::Other;\n', BASE_MAIN, {}),
    ('workspace', BASE_LIB, BASE_MAIN, {'Cargo.toml': MANIFEST.replace('[workspace]', '[workspace]\nmembers=["other"]')}),
    ('unaccounted', BASE_LIB, BASE_MAIN, {'src/forgotten.rs': 'pub fn unused() {}\n'}),
    ('duplicate', BASE_LIB, 'struct Parcel; fn main() { let _ = Parcel; }\n', {}),
    ('rollback', BASE_LIB, BASE_MAIN.replace('let p', 'missing(); let p'), {}),
    ('doc_macro', '#[doc=include_str!("note")] '+BASE_LIB, BASE_MAIN, {}),
]:
    files = {'src/lib.rs': (lib, lib), 'src/main.rs': (main, main)}
    files.update({k:(v,v) for k,v in extra.items()})
    CASES.append((name, 'Parcel', 'Packet', False, files))


def run(command, root, env):
    return subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, timeout=620)


def cargo(root, env, *args):
    assert env['TMPDIR'] and not env['TMPDIR'].startswith('/tmp')
    command = ['systemd-run', '--user', '--scope', '-pMemoryMax=8G', '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout', '--kill-after=10s', '600', 'cargo', *args, '-j1']
    return run(command, root, env)


def environment(case):
    env = {k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
    target = Path(env['CARGO_TARGET_DIR']) / 'package-fixtures' / case
    env.update(CARGO_TARGET_DIR=str(target), GLASSMIND_CARGO_TARGET=str(target / 'checks'), GLASSMIND_CHECK_CACHE=str(Path(env['TMPDIR']) / 'package-cache' / case), GLASSMIND_SCRATCH=str(Path(env['TMPDIR']) / 'package-scratch' / case))
    return env


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--sample', action='store_true')
    p.add_argument('--dev-inputs', type=Path)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    rows = []
    if a.dev_inputs:
        for name in ['rename-item-struct', 'rename-done-variant']:
            prompts = json.loads((a.dev_inputs / (name+'-prompts.json')).read_text())
            for style, prompt in prompts.items():
                root = a.output / (name+'-'+style)
                shutil.copytree(a.dev_inputs/name, root)
                env = environment(a.output.name+'-'+root.name)
                before = {str(f.relative_to(root)):f.read_bytes() for f in root.rglob('*.rs')}
                pre = cargo(root, env, 'check', '--all-targets')
                done = run([str(a.binary.resolve()), 'do', str(root.resolve()), prompt], root, env)
                post = cargo(root, env, 'check', '--all-targets')
                changed = [f for f,b in before.items() if (root/f).read_bytes()!=b]
                row = dict(case=root.name, prompt=prompt, changed=changed, before_exit=pre.returncode, after_exit=post.returncode, reply=done.stdout, stderr=done.stderr, edit_exit=done.returncode, check_stderr=post.stderr)
                rows.append(row)
                print(json.dumps(row), flush=True)
    else:
        for name, old, new, supported, files in (CASES[:3] if a.sample else CASES):
            root = a.output / name
            root.mkdir()
            (root/'Cargo.toml').write_text(MANIFEST)
            for path,(before,_) in files.items():
                (root/path).parent.mkdir(parents=True,exist_ok=True)
                (root/path).write_text(before)
            env = environment(a.output.name+'-'+name)
            before = cargo(root,env,'run','--quiet')
            done = run([str(a.binary.resolve()),'do',str(root.resolve()),f'Rename `{old}` to `{new}`'],root,env)
            changed = any((root/f).read_text()!=pair[0] for f,pair in files.items())
            after = cargo(root,env,'run','--quiet') if changed else before
            exact = all((root/f).read_text()==pair[1] for f,pair in files.items())
            behavior = before.returncode==after.returncode==0 and before.stdout==after.stdout
            row = dict(case=name, supported=supported, changed=changed, exact_source=exact, behavior=behavior, before_exit=before.returncode, after_exit=after.returncode, reply=done.stdout, stderr=done.stderr, edit_exit=done.returncode, before_stdout=before.stdout, after_stdout=after.stdout, before_stderr=before.stderr, after_stderr=after.stderr)
            row['pass'] = exact and (changed and behavior if supported else not changed)
            rows.append(row)
            print(json.dumps(row),flush=True)
    report = dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows)
    (a.output/'results.json').write_text(json.dumps(report,indent=2)+'\n')

if __name__=='__main__':
    main()
