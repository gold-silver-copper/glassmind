#!/usr/bin/env python3
"""Independent authored development probes v1; MIT OR Apache-2.0.

Freeze first; run --sample before the full run. No archived generator imports.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
RUN = Path('/home/gsc/.cache/glassmind/continuous-20261010/fieldreview-run')
ARCHIVE = Path('/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run')
SCOPE = ['systemd-run', '--user', '--scope', '-pMemoryMax=8G', '-pMemorySwapMax=0',
         'setsid', '--wait', 'timeout', '--kill-after=10s', '600']
BINARIES = {'baseline': RUN / 'baseline/glassmind', 'accepted': ARCHIVE / 'glassmind-accepted'}
HASHES = {'baseline': 'ca120ec676a8979d9116ef10380e58cd6f7f3d033fc97ade968a7d510c5b1bd7',
          'accepted': '19d2770f8a43cdd9037b8a46747d5a428489b567dcdfd63e40e1bc1e011fb621'}


def digest(b):
    return hashlib.sha256(b).hexdigest()


def cases():
    out = []
    # Only explicitly marked target-owner occurrences vary in expected sources.
    # The unrelated owner's occurrences, comments, strings and assertions do not.
    def add(name, files, accepted=True, new='reading'):
        test = ('\n#[test] fn both_owners() -> Result<(), String> {\n'
                f'    println!("REVIEW_BEHAVIOR_{name}");\n'
                '    if observe() != (13, 71) { return Err("owners changed".into()); }\n'
                '    Ok(())\n}\n')
        files = dict(files)
        files['src/lib.rs'] += test
        before = {k: v.replace('@', 'level') for k, v in files.items()}
        expected = {k: v.replace('@', new) for k, v in files.items()} if accepted else before
        out.append(dict(name=name, before=before, expected=expected, accepted=accepted,
                        request=f'rename field `level` on `Gauge` to `{new}`'))

    prefix = 'pub struct Gauge { pub @: u32 }\npub struct Beacon { pub level: u32 }\n'
    simple = prefix + ('pub fn observe() -> (u32,u32) { '
                       '(Gauge { @: 13 }.@, Beacon { level: 71 }.level) }\n')
    add('nested_literal', {'src/lib.rs': prefix +
        'fn pair(g: &&Gauge, b: &Beacon) -> (u32,u32) { ((g).@, (b).level) }\n'
        'pub fn observe() -> (u32,u32) { pair(&&Gauge { @: Beacon { level: 13 }.level }, '
        '&Beacon { level: Gauge { @: 71 }.@ }) }\n'})
    add('qualified_files', {
        'src/lib.rs': 'pub mod metrics;\npub struct Beacon { pub level: u32 }\n'
            'pub fn observe() -> (u32,u32) { metrics::inner::read(&metrics::Gauge { @: 13 }, '
            '&crate::Beacon { level: 71 }) }\n',
        'src/metrics.rs': 'pub struct Gauge { pub @: u32 }\npub mod inner {\n'
            'pub fn read(g: &super::Gauge, b: &crate::Beacon) -> (u32,u32) { (g.@, b.level) }\n}\n'})
    add('struct_update', {'src/lib.rs': prefix +
        'fn read(g: Gauge, b: Beacon) -> (u32,u32) { '
        '(Gauge { ..g }.@, Beacon { ..b }.level) }\n'
        'pub fn observe() -> (u32,u32) { read(Gauge { @: 13 }, Beacon { level: 71 }) }\n'})
    add('unicode_crlf', {'src/lib.rs': (prefix +
        'pub fn observe() -> (u32,u32) { let café = "🌡 level"; let _ = café; '
        '(Gauge { @: 13 }.@, Beacon { level: 71 }.level) }\n').replace('\n', '\r\n')})
    add('match_shadow', {'src/lib.rs': prefix +
        'fn read(g: Gauge, b: Beacon) -> (u32,u32) { let first = g.@; '
        'match b { g => (first, g.level) } }\n'
        'pub fn observe() -> (u32,u32) { read(Gauge { @: 13 }, Beacon { level: 71 }) }\n'}, False)
    add('qualified_member', {'src/lib.rs': prefix +
        'pub struct Holder { pub inner: Gauge }\n'
        'pub fn observe() -> (u32,u32) { '
        '(Holder { inner: Gauge { @: 13 } }.inner.@, Beacon { level: 71 }.level) }\n'}, False)
    add('duplicate_identity', {'src/lib.rs': simple +
        'pub mod extra { pub struct Gauge { pub spare: u32 } }\n'}, False)
    add('macro_token', {'src/lib.rs': simple +
        'pub fn extra(g: Gauge) { println!("{}", g.@); }\n'}, False)
    add('cfg_attribute', {'src/lib.rs': simple +
        '#[cfg(any())] pub fn disabled() {}\n'}, False)
    add('uncovered_source', {'src/lib.rs': simple, 'unattached.rs':
        'pub fn unattached() -> u32 { 5 }\n'}, False)
    add('new_collision', {'src/lib.rs': simple + 'pub const reading: u32 = 9;\n'}, False)
    add('raw_spelling', {'src/lib.rs': simple.replace('}.@', '}.r#level')}, False)
    add('alias_parameter', {'src/lib.rs': prefix +
        'type LocalGauge = Gauge;\nfn read(g: LocalGauge) -> u32 { g.@ }\n'
        'pub fn observe() -> (u32,u32) { (read(Gauge { @: 13 }), Beacon { level: 71 }.level) }\n'}, False)
    add('generic_parameter', {'src/lib.rs': simple +
        'pub fn untouched<Gauge>(g: Gauge) -> Gauge { g }\n'}, False)
    # Unique-spelling route must retain the original behavior on both binaries.
    add('inherited_unique', {'src/lib.rs': simple.replace('Beacon { pub level:', 'Beacon { pub depth:')
        .replace('Beacon { level: 71 }.level', 'Beacon { depth: 71 }.depth')})
    out[-1]['baseline_accept'] = True
    return out


def freeze():
    frozen = HERE / 'frozen.json'
    data = json.dumps(cases(), ensure_ascii=False, indent=2).encode() + b'\n'
    if frozen.exists():
        assert frozen.read_bytes() == data, 'Frozen expectations changed'
    else:
        frozen.write_bytes(data)
    print('frozen', digest(data), flush=True)


def run(binary, sample, tag=''):
    freeze()
    assert digest(BINARIES[binary].read_bytes()) == HASHES[binary]
    env = {k: v for k, v in os.environ.items() if not k.endswith('API_KEY')}
    env['TMPDIR'] = str(RUN / 'tmp')
    env['GLASSMIND_SCRATCH'] = str(RUN / 'scratch')
    env['CARGO_BUILD_JOBS'] = '1'
    assert env['CARGO_TARGET_DIR'] == str(RUN / 'target')
    for key in ('TMPDIR', 'GLASSMIND_SCRATCH'):
        Path(env[key]).mkdir(parents=True, exist_ok=True)
    assert all(c.isalnum() or c in '-_' for c in tag)
    label = binary + ('-sample' if sample else '-full') + ('-' + tag if tag else '')
    results = HERE / 'reports' / label
    results.mkdir(parents=True, exist_ok=True)
    rows = []
    frozen = json.loads((HERE / 'frozen.json').read_bytes())
    for case in frozen[:3] if sample else frozen:
        root = RUN / 'probes' / label / case['name']
        root.mkdir(parents=True, exist_ok=False)
        manifest = '[package]\nname="field_review_probe"\nversion="0.0.0"\nedition="2021"\n[workspace]\n'
        (root / 'Cargo.toml').write_text(manifest)
        for filename, code in case['before'].items():
            path = root / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(code.encode())
        def command(args):
            p = subprocess.run(SCOPE + list(map(str, args)), cwd=root, env=env,
                               capture_output=True, text=True, timeout=620)
            return dict(code=p.returncode, stdout=p.stdout, stderr=p.stderr)
        test_args = ['cargo', 'test', '--offline', '-j1', '--', '--test-threads', '4', '--nocapture']
        before = command(test_args)
        valid = before['code'] == 0 and '1 passed' in before['stdout']
        edit = command([BINARIES[binary], 'do', root, case['request']]) if valid else None
        after_files = {p: (root / p).read_bytes().decode() for p in case['before']}
        after = command(test_args) if valid else None
        expected = case['expected'] if binary == 'accepted' or case.get('baseline_accept') else case['before']
        marker = 'REVIEW_BEHAVIOR_' + case['name']
        behavior = valid and all(r['code'] == 0 and '1 passed' in r['stdout'] and marker in r['stdout']
                                 for r in [before, after])
        row = dict(name=case['name'], valid_original=valid, exact=after_files == expected,
                   changed=after_files != case['before'], behavior=behavior,
                   manifest_unchanged=(root / 'Cargo.toml').read_text() == manifest,
                   before=before, edit=edit, after=after)
        row['pass'] = row['exact'] and row['behavior'] and row['manifest_unchanged']
        record = results / case['name']
        record.mkdir(exist_ok=True)
        for stage, files in [('before', case['before']), ('expected', expected), ('after', after_files)]:
            for filename, code in files.items():
                path = record / stage / filename
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(code.encode())
        (record / 'result.json').write_text(json.dumps(row, indent=2) + '\n')
        rows.append(row)
        (results / 'results.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(label, case['name'], 'valid', valid, 'changed', row['changed'],
              'exact', row['exact'], 'behavior', behavior, flush=True)
    print('passed', sum(r['pass'] for r in rows), '/', len(rows), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--freeze', action='store_true')
    parser.add_argument('--binary', choices=BINARIES)
    parser.add_argument('--sample', action='store_true')
    parser.add_argument('--tag', default='', help='Fresh suffix for non-destructive replays')
    args = parser.parse_args()
    if args.freeze:
        freeze()
    else:
        run(args.binary, args.sample, args.tag)
