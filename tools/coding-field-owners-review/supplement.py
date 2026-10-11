#!/usr/bin/env python3
"""Additional frozen path/binding contrasts, authored v1, MIT OR Apache-2.0."""
import argparse
import probe


def cases():
    base = probe_cases()[0]
    source = ('pub struct Gauge { pub @: u32 }\npub struct Beacon { pub level: u32 }\n'
              'pub fn observe() -> (u32,u32) { (Gauge { @: 13 }.@, Beacon { level: 71 }.level) }\n')
    output = []
    def add(name, text, supported, extra=None):
        files = {'src/lib.rs': text +
                 f'#[test] fn both_owners() -> Result<(), String> {{ println!("REVIEW_BEHAVIOR_{name}"); '
                 'if observe() != (13, 71) { return Err("owners changed".into()); } Ok(()) }\n'}
        files.update(extra or {})
        before = {k: v.replace('@', 'level') for k, v in files.items()}
        expected = {k: v.replace('@', 'reading') for k, v in files.items()} if supported else before
        output.append(dict(name=name, before=before, expected=expected, accepted=supported,
                           request=base['request']))
    add('lib_bin_paths', source, True, {'src/main.rs':
        'fn observe() -> (u32,u32) { (field_review_probe::Gauge { @: 13 }.@, '
        'field_review_probe::Beacon { level: 71 }.level) }\n'
        'fn main() { let _ = observe(); }\n'
        '#[test] fn both_owners() -> Result<(), String> { println!("REVIEW_BEHAVIOR_lib_bin_paths"); '
        'if observe() != (13, 71) { return Err("bin owners changed".into()); } Ok(()) }\n'})
    add('for_shadow', source +
        'pub fn loop_value(g: Gauge) -> u32 { let mut n = g.@; '
        'for g in [Beacon { level: 1 }] { n += g.level; } n }\n', False)
    add('raw_receiver_binding', source +
        'pub fn read(r#g: Gauge) -> u32 { g.@ }\n', False)
    add('destructure_member', source +
        'pub fn read(g: Gauge) -> u32 { let Gauge { @: n } = g; n }\n', False)
    add('enum_variant_collision', source +
        'pub enum Signal { Pulse { level: u32 } }\n', False)
    add('unrelated_attribute', source + '#[allow(dead_code)] pub fn idle() {}\n', False)
    return output


probe_cases = probe.cases
probe.cases = cases
probe.HERE = probe.HERE / 'supplemental'
probe.HERE.mkdir(exist_ok=True)
# Every supplemental case has a distinct name, also avoiding fixture-root overlap.

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', action='store_true')
    p.add_argument('--binary', choices=probe.BINARIES)
    p.add_argument('--tag', default='')
    a = p.parse_args()
    if a.freeze:
        probe.freeze()
    else:
        probe.run(a.binary, False, a.tag)
