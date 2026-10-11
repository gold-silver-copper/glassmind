#!/usr/bin/env python3
"""Independent declared-source-type probes. No cargo, hidden data, or network.

Fixtures authored for this batch, MIT OR Apache-2.0. The original seven-request
replay and its export definitions remain in coding-answerfiles-dev.py.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

spec = importlib.util.spec_from_file_location('exports', Path(__file__).with_name('coding-answerfiles-dev.py'))
exports = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exports)

MANIFEST = '[package]\nname="type-probes"\nversion="0.1.0"\nedition="2021"\n'
FORMS = [
    'Vec < Option < u32 > >', "&'a mut [u16]", '(u8, bool,)',
    '[u8; 2 + 3]', '<T as Iterator>::Item', 'crate::nested::Handle<T>',
    'Alias', 'Option<\n        /* keep this */ Vec<u8>\n    >',
    'fn(u8, &str) -> bool', '*const u8', "&'a (dyn Send + Sync)",
]
BASE = "pub type Alias = u64;\npub struct CrateBox<'a, T: Iterator> { pub payload: TYPE }\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--sample', action='store_true')
    args = parser.parse_args()
    assert not str(args.output.resolve()).startswith('/tmp/')
    args.output.mkdir(parents=True, exist_ok=False)
    args.binary = args.binary.resolve()
    cases = [(f'form-{i}', BASE.replace('TYPE', form), 'CrateBox.payload', form, None) for i, form in enumerate(FORMS)]
    cases += [
        ('unicode', 'pub struct Café { pub naïve: Vec < u16 > }', 'Café.naïve', 'Vec < u16 >', None),
        ('other-owner', BASE.replace('TYPE', 'Alias')+'pub struct Another { pub payload: bool }', 'CrateBox.payload', 'Alias', None),
        ('duplicate-owner', BASE.replace('TYPE', 'u8')+'mod x { pub struct CrateBox { pub other: bool } }', 'CrateBox.payload', None, None),
        ('alias-owner-collision', BASE.replace('TYPE', 'u8')+'mod x { type CrateBox = u64; }', 'CrateBox.payload', None, None),
        ('duplicate-field', 'struct CrateBox { payload: u8, payload: bool }', 'CrateBox.payload', None, None),
        ('tuple-index', 'struct CrateBox(u8);', 'CrateBox.0', None, None),
        ('unknown-owner', 'struct Other { payload: u8 }', 'CrateBox.payload', None, None),
        ('missing-field', 'struct CrateBox { other: u8 }', 'CrateBox.payload', None, None),
        ('malformed', 'struct CrateBox { payload: }', 'CrateBox.payload', None, None),
        ('cfg', '#[cfg(feature="x")] struct CrateBox { payload: u8 }', 'CrateBox.payload', None, None),
        ('parent-cfg', '#[cfg(feature="x")] mod inner { struct CrateBox { payload: u8 } }', 'CrateBox.payload', None, None),
        ('macro-generated', 'macro_rules! make { () => { struct CrateBox { payload: u8 } } } make!();', 'CrateBox.payload', None, None),
        ('custom-attribute', '#[transform] struct CrateBox { payload: u8 }', 'CrateBox.payload', None, None),
        ('type-macro', 'struct CrateBox { payload: some_type!() }', 'CrateBox.payload', None, None),
    ]
    for name in ['occupied', 'symlink', 'parent-symlink', 'traversal', 'source', 'extra-action', 'irrelevant', 'bad-directive', 'format', 'transcript-alias']:
        cases.append((name, BASE.replace('TYPE', 'Alias'), 'CrateBox.payload', None, name))
    rows = []
    for name, source, reference, expected, unsafe in cases[:3] if args.sample else cases:
        root = args.output / name
        (root / 'src/nested').mkdir(parents=True)
        (root / 'reports/deeper').mkdir(parents=True)
        (root / 'Cargo.toml').write_text(MANIFEST)
        (root / 'src/lib.rs').write_text('mod nested { pub mod payload; }\n')
        (root / 'src/nested/payload.rs').write_text(source)
        dest = 'reports/deeper/type.txt'
        outside = args.output / (name+'-sentinel')
        outside.write_text('preserve')
        if unsafe == 'occupied': (root / dest).write_text('preserve')
        if unsafe == 'symlink': (root / dest).symlink_to(outside.resolve())
        if unsafe == 'parent-symlink':
            (root / 'linked').symlink_to(args.output.resolve(), target_is_directory=True)
            dest = 'linked/escape'
        if unsafe == 'traversal': dest = '../escape'
        if unsafe == 'source': dest = 'src/nested/payload.rs'
        question = f'What is the type of {reference}?'
        if unsafe == 'extra-action': question = f'What is the type of {reference} and rename CrateBox to Wrong?'
        if unsafe == 'irrelevant': question = f'What is the type of {reference} irrelevant?'
        text = question + f' Save the type to {dest}.'
        if unsafe == 'bad-directive': text += ' Fix the build.'
        if unsafe == 'format': text = question + f' Save the number to {dest}.'
        transcript = root / dest if unsafe == 'transcript-alias' else None
        row = exports.run(args.binary, root, text, dest, None if expected is None else expected+'\n', transcript)
        row['safe'] &= outside.read_text() == 'preserve' and not (args.output / 'escape').exists()
        row['passed'] = row['safe'] and (row['usable'] if expected is not None else not row['usable'])
        # Independently verify ordinary typed JSON and provenance offsets, including Unicode.
        before = exports.snapshot(root)
        result = subprocess.run([str(args.binary), 'ask', '-s', str(root), '--json', question],
                                capture_output=True, text=True, timeout=30,
                                env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')})
        row['ordinary_stdout'] = result.stdout
        if unsafe is None:
            try:
                answer = json.loads(result.stdout)
                if expected is None:
                    ordinary = 'refused' in answer
                else:
                    typed = answer['declared_type']
                    data = (root / typed['field']['file']).read_bytes()
                    ordinary = typed['spelling'] == expected and data[typed['start']:typed['end']].decode() == expected
            except (ValueError, KeyError):
                ordinary = False
            row['passed'] &= ordinary
        row['passed'] &= before == exports.snapshot(root)
        if name == 'form-0':
            # The plain do reply and export trace must preserve transcript semantics.
            transcript = args.output / 'ordinary-type-transcript.txt'
            ordinary = subprocess.run([str(args.binary), 'do', str(root), '--explain', '--reply-file', str(transcript), question], capture_output=True, text=True, timeout=30)
            trace = all(part in ordinary.stdout for part in ['Metadata question:', 'Resolved answer:', 'World rule:'])
            row['ordinary_explain'] = ordinary.stdout
            row['passed'] &= ordinary.returncode == 0 and trace and transcript.exists() and expected in transcript.read_text() and 'source spelling, not resolved' in transcript.read_text()
            trace_dest = root / 'reports/deeper/explained.txt'
            export = subprocess.run([str(args.binary), 'do', str(root), '--explain', '--reply-file', str(transcript), question+' Write the type to reports/deeper/explained.txt.'], capture_output=True, text=True, timeout=30)
            row['export_explain'] = export.stdout
            row['passed'] &= export.returncode == 0 and trace_dest.exists() and trace_dest.read_text() == expected+'\n' and all(part in export.stdout for part in ['Answer export:', 'Metadata question:', 'Resolved answer:', 'World rule:'])
            row['passed'] &= transcript.read_text() == f'Wrote {expected} to reports/deeper/explained.txt.\n'
            after_trace = exports.snapshot(root)
            row['passed'] &= all(after_trace.get(p) == value for p, value in before.items())
        rows.append(row)
    report = dict(total=len(rows), passed=sum(r['passed'] for r in rows),
                  usable=sum(r['usable'] for r in rows), safe=sum(r['safe'] for r in rows),
                  fixture_bytes=sum(sum(p.stat().st_size for p in (args.output/r['case']).rglob('*') if p.is_file() and p.name in ['Cargo.toml', 'lib.rs', 'payload.rs']) for r in rows),
                  binary_sha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(), rows=rows)
    (args.output / 'results.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k != 'rows'}))
    for row in rows:
        if not row['passed']: print(row['case'], row['stdout'], row['ordinary_stdout'])
    return 0 if report['passed'] == report['total'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
