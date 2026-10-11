#!/usr/bin/env python3
"""Independent diagnostic probes, not additional official benchmark items.
Original fixtures and checks: MIT OR Apache-2.0, same as glassmind.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('dev', Path(__file__).resolve().parents[1] / 'coding-addfields-dev.py')
dev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dev)

CASES = [
    ('polite-reordered', 'Sensor', 'offset', 'i32', '-12',
     'Use -12 for all existing constructors. Could you please add a public field offset: i32 to the Sensor struct?', True),
    ('hurried-bool', 'Beacon', 'active', 'bool', 'true',
     'add active: bool (pub) to Beacon struct + fix everywhere beacons get created, just use true active for the existing ones', True),
    ('partition-char', 'Rune', 'marker', 'char', "'λ'",
     "need a marker field on Rune (pub, char). default 'λ' in Rune::new and use 'λ' for all the other Rune { .. } literals", True),
    ('contraction-context', 'Ledger', 'code', 'u64', '0x2A_u64',
     "Hi! I'd like you to add pub code: u64 to Ledger. I don't know the default yet. Use 0x2A_u64 for every existing constructor. Thanks!", True),
    ('scope-first', 'Crate', 'depth', 'usize', '5',
     'Update every place a Crate is built. Use 5 for the existing crates. Add a pub depth: usize field to Crate.', True),
    ('literal-backtick', 'Clock', 'tick', 'char', "'`'",
     "Add pub tick: char to Clock. Use '`' for all existing constructors.", True),
    ('negative-compound', 'Sensor', 'offset', 'i32', '-12',
     'Add pub offset: i32 to Sensor. Use -12 for all existing constructors and rename Sensor to Gauge.', False),
    ('negative-negation', 'Sensor', 'offset', 'i32', '-12',
     "Don't add pub offset: i32 to Sensor. Use -12 for all existing constructors.", False),
    ('negative-conflict', 'Sensor', 'offset', 'i32', '-12',
     'Add pub offset: i32 to Sensor. Sensor::new sets it to -12; every other place that builds a Sensor uses -13 too.', False),
    ('negative-owner', 'Sensor', 'offset', 'i32', '-12',
     'Use -12 for all existing constructors. Add pub offset: i32 to Sensor. Update every place a Gauge is built.', False),
    ('negative-unconsumed', 'Sensor', 'offset', 'i32', '-12',
     'Could you add pub offset: i32 to Sensor? Use -12 for all existing constructors except tests.', False),
    ('negative-context-action', 'Sensor', 'offset', 'i32', '-12',
     "I don't know the default yet and delete the tests. Add pub offset: i32 to Sensor. Use -12 for all existing constructors.", False),
    ('negative-call', 'Sensor', 'offset', 'i32', '-12',
     'Add pub offset: i32 to Sensor. Use 12() for all existing constructors.', False),
]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--binary', type=Path, required=True)
    ap.add_argument('--output', type=Path, required=True)
    a = ap.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    rows = []
    for name, owner, field, ty, value, request, positive in CASES:
        root = a.output / name
        (root/'src').mkdir(parents=True)
        (root/'Cargo.toml').write_text('[package]\nname="independent"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
        (root/'src/lib.rs').write_text(f'pub struct {owner} {{ pub old: u8 }}\nimpl {owner} {{ pub fn new() -> Self {{ Self {{ old: 9 }} }} }}\n#[test] fn original() {{ assert_eq!({owner}::new().old, 9); }}\n')
        (root/'src/main.rs').write_text(f'use independent::{owner} as Item; fn main() {{ let p=Item {{ old: 2 }}; println!("{{}} {{}}",p.old, Item::new().old); }}\n')
        before = dev.snapshot(root)
        old_behavior = dev.cargo(root, 'run')
        result = dev.run([a.binary.resolve(), 'do', root, request])
        after = dev.snapshot(root)
        changed = before != after
        row = dict(case=name, request=request, expected_edit=positive, changed=changed, result=result)
        if changed and positive:
            declaration = f' pub {field}: {ty},'.encode()
            default = f' {field}: {value},'.encode()
            preserved = before.keys() == after.keys() and all(after[k].replace(declaration,b'').replace(default,b'') == v for k,v in before.items())
            (root/'tests').mkdir()
            (root/'tests/contract.rs').write_text(f'#[test] fn contract() {{ let p=independent::{owner}::new(); let _: {ty} = p.{field}; assert_eq!(p.{field}, {value}); assert_eq!(p.old,9); }}')
            tests = dev.cargo(root, 'test')
            behavior = dev.cargo(root, 'run')
            row.update(preserved=preserved, tests=tests, behavior=behavior, old_behavior=old_behavior)
            row['passed'] = preserved and tests['code']==0 and old_behavior['code']==0 and behavior['code']==0 and behavior['stdout']==old_behavior['stdout']
        else:
            row['passed'] = not positive and not changed
        rows.append(row)
    (a.output/'results.json').write_text(json.dumps(rows, indent=2)+'\n')
    print(json.dumps(dict(cases=len(rows), passed=sum(r['passed'] for r in rows), changed=sum(r['changed'] for r in rows))))
    raise SystemExit(0 if all(r['passed'] for r in rows) else 1)
if __name__ == '__main__': main()
