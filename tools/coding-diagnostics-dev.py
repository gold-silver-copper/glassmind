#!/usr/bin/env python3
"""Permitted workspace-only diagnostic replay; no benchmark labels or hidden tests.

Run under the prescribed systemd scope. Functional probes are authored here and
added only after repair, with fixed outputs. All generated artifacts stay in the
provided TMPDIR and CARGO_TARGET_DIR.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

SAMPLE = ['derive-debug', 'collect-needs-type', 'match-missing-variant']
FULL = SAMPLE + ['copy-struct', 'int-float-divide', 'private-fn']
DEBUG_OUTPUT = '[Reading { sensor: "attic", celsius: 18.5 }, Reading { sensor: "cellar", celsius: 12.0 }]\n'
FUNCTIONAL = '''fn main() {
    use task::{Reading, warmest};
    let unequal = [Reading { sensor: "cold".into(), celsius: -3.0 }, Reading { sensor: "hot".into(), celsius: 2.5 }];
    let single = [Reading { sensor: "first".into(), celsius: 0.0 }];
    println!("{:?}|{:?}|{:?}", warmest(&[]), warmest(&unequal), warmest(&single));
}
'''
MANIFEST = '[package]\nname="diagnostic_probe"\nversion="0.0.0"\nedition="2021"\n[workspace]\n'
# Independent visible development fixtures, not benchmark or generated labels.
FIXTURES = [
    ('record', 'pub struct Packet { pub n: u32 }', 'use diagnostic_probe::Packet; fn main() { println!("{:?}", Packet { n: 17 }); }', 'Packet { n: 17 }\n'),
    ('tuple_unicode', 'const É: u8 = 1; pub struct État(pub u8);', 'use diagnostic_probe::État; fn main() { println!("é {:?}", État(7)); }', 'é État(7)\n'),
    ('enum', 'pub enum Mode { Idle, Ready(u8) }', 'use diagnostic_probe::Mode; fn main() { println!("{:?} {:?}", Mode::Idle, Mode::Ready(4)); }', 'Idle Ready(4)\n'),
    ('nested', 'pub mod data { pub struct Event(pub bool); }', 'use diagnostic_probe::data::Event; fn main() { println!("{:?}", Event(true)); }', 'Event(true)\n'),
    ('ambiguous', 'pub mod a { pub struct Item; } pub mod b { pub struct Item; }', 'use diagnostic_probe::a::Item; fn main() { println!("{:?}", Item); }', None),
    ('conditional', '#[cfg(unix)] pub struct Item;', 'use diagnostic_probe::Item; fn main() { println!("{:?}", Item); }', None),
    ('unrelated_error', 'pub struct Item;', 'use diagnostic_probe::Item; fn main() { println!("{:?}", Item); absent(); }', None),
    ('non_debug_field', 'pub struct Item(pub Box<dyn Fn()>);', 'use diagnostic_probe::Item; fn main() { println!("{:?}", Item(Box::new(|| {}))); }', None),
]


def invoke(command, root, env):
    return subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, timeout=620)


def cargo(root, env, *args):
    return invoke(['systemd-run', '--user', '--scope', '-pMemoryMax=8G',
                   '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout',
                   '--kill-after=10s', '600', 'cargo', *args, '-j1'], root, env)


def snapshot(root):
    return {str(p.relative_to(root)):p.read_bytes() for p in root.rglob('*.rs')}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--dev-inputs', type=Path)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--sample', action='store_true')
    a = p.parse_args()
    output = a.output.resolve()
    assert output.is_relative_to(Path(os.environ['TMPDIR']).resolve())
    output.mkdir(parents=True, exist_ok=False)
    cases = []
    if a.dev_inputs:
        for name in SAMPLE if a.sample else FULL:
            prompts = json.loads((a.dev_inputs/(name+'-prompts.json')).read_text())
            for style, prompt in prompts.items():
                if a.sample and style != 'terse':
                    continue
                cases.append((name+'-'+style, a.dev_inputs/name, prompt, None))
    else:
        cases = [(name, (lib, main), 'fix the build', expected) for name,lib,main,expected in FIXTURES]
    rows = []
    for name, source, prompt, expected in cases:
        root = output/name
        if isinstance(source, Path):
            shutil.copytree(source, root)
        else:
            (root/'src').mkdir(parents=True)
            (root/'Cargo.toml').write_text(MANIFEST)
            (root/'src/lib.rs').write_text(source[0])
            (root/'src/main.rs').write_text(source[1])
        env = {k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
        target = Path(env['CARGO_TARGET_DIR'])/'diagnostics'/output.name/name
        env.update(CARGO_TARGET_DIR=str(target/'verify'), GLASSMIND_CARGO_TARGET=str(target/'checks'),
                   GLASSMIND_SCRATCH=str(output/'scratch'), GLASSMIND_CHECK_CACHE=str(output/'cache'/name))
        before = snapshot(root)
        pre = cargo(root, env, 'check', '--all-targets', '--message-format=json')
        (output/(name+'-before.jsonl')).write_text(pre.stdout)
        done = invoke([str(a.binary.resolve()), 'do', str(root), prompt, '--explain'], root, env)
        changed = snapshot(root) != before
        post = cargo(root, env, 'check', '--all-targets', '--message-format=json') if changed else pre
        functional = None
        if changed and name.startswith('derive-debug-'):
            run = cargo(root, env, 'run', '--quiet')
            (root/'examples').mkdir(exist_ok=True)
            (root/'examples/functional_probe.rs').write_text(FUNCTIONAL)
            probe = cargo(root, env, 'run', '--quiet', '--example', 'functional_probe')
            functional = run.returncode == probe.returncode == 0 and run.stdout == DEBUG_OUTPUT and probe.stdout == 'None|Some("hot")|Some("first")\n'
        elif changed and expected is not None:
            run = cargo(root, env, 'run', '--quiet')
            functional = run.returncode == 0 and run.stdout == expected
        row = dict(case=name, before_exit=pre.returncode, after_exit=post.returncode,
                   changed=changed, functional=functional, reply=done.stdout, stderr=done.stderr,
                   command_exit=done.returncode, after_diagnostics=post.stdout)
        row['verdict'] = ('verified' if post.returncode == 0 and functional is True else 'wrong') if changed else 'refused'
        if not a.dev_inputs:
            row['fixture_pass'] = row['verdict'] == ('verified' if expected is not None else 'refused')
        rows.append(row)
        print(name, row['verdict'], flush=True)
        report = dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), cases=rows)
        (output/'results.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
