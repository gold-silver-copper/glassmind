#!/usr/bin/env python3
"""Source-only dev replay and independent metadata export safety fixtures.

Does not read task manifests, solutions, hidden tests, or invoke cargo.
All subprocesses finish before returning. Run outside /tmp.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

# Independently counted from supplied production source, not benchmark labels.
DEV_VALUES = {'callers-of-apply-damage': '3\n', 'command-variant-count': '6\n',
              'items-field-type': 'Vec<LineItem>\n'}
MANIFEST = '[package]\nname="independent-metadata"\nversion="0.0.0"\nedition="2021"\n'
SOURCE = '''pub enum Signal { Open, Data(u8), Close }
pub fn pulse() {}
pub fn dispatch() { pulse(); pulse(); }
pub fn unused() {}
pub struct Basket { pub values: Vec<u16> }
'''


def snapshot(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob('*') if p.is_file() and not p.is_symlink()}


def run(binary, root, text, destination, expected, transcript=None):
    before = snapshot(root)
    transcript = transcript or root.parent / (root.name + '-transcript.txt')
    env = {k: v for k, v in os.environ.items() if not k.endswith('API_KEY')}
    done = subprocess.run([str(binary), 'do', str(root), '--reply-file', str(transcript), text],
                          capture_output=True, text=True, timeout=30, env=env)
    path = root / destination
    answer = path.read_text() if path.is_file() and not path.is_symlink() else None
    after = snapshot(root)
    mutations = [p for p in before if after.get(p) != before[p]]
    added = sorted(set(after) - set(before))
    usable = expected is not None and answer == expected and destination in added
    safe = not mutations and (added == [destination] if usable else not added)
    return dict(case=root.name, prompt=text, expected=expected, answer=answer,
                usable=usable, safe=safe, changed=mutations, added=added,
                returncode=done.returncode, stdout=done.stdout, stderr=done.stderr,
                transcript=transcript.read_text() if transcript.is_file() else None,
                before=before, after=after)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--binary', required=True, type=Path)
    p.add_argument('--output', required=True, type=Path)
    p.add_argument('--dev-inputs', type=Path)
    p.add_argument('--sample', action='store_true')
    a = p.parse_args()
    a.binary = a.binary.resolve()
    a.output = a.output.resolve()
    assert not str(a.output).startswith('/tmp/')
    a.output.mkdir(parents=True, exist_ok=False)
    rows = []
    if a.dev_inputs:
        for name, value in DEV_VALUES.items():
            source = a.dev_inputs / name
            for request in json.loads((source / 'prompts.json').read_text()):
                if a.sample and request['style'] != 'terse':
                    continue
                root = a.output / (name + '-' + request['style'])
                shutil.copytree(source / 'workspace', root)
                rows.append(run(a.binary, root, request['text'], 'ANSWER', value))
    else:
        count = 'How many variants of `Signal`?'
        callers = 'How many functions call `pulse`?'
        cases = [
            ('count', count+' Write just the number to a file named tally.txt.', 'tally.txt', '3\n', None),
            ('caller_count', callers+' Put only the number into reports/call-count.', 'reports/call-count', '1\n', None),
            ('unique_name', 'Which functions call `pulse`? Save the name to a file named caller-id.', 'caller-id', 'dispatch\n', None),
            ('have_members', 'How many variants does the `Signal` enum have? Write the number to size.', 'size', '3\n', None),
            ('quoted_filename', count+' Write the number to "résultat-count.txt".', 'résultat-count.txt', '3\n', None),
            ('unclosed_quote', count+' Write the number to `unsafe.', 'unsafe', None, None),
            ('zero', 'How many functions call `unused`? Write the number to zero.', 'zero', '0\n', None),
            ('unknown', 'How many functions call `missing`? Write the number to unknown.', 'unknown', None, None),
            ('ambiguous', callers+' Write the number to ambiguous.', 'ambiguous', None, 'mod other { pub fn pulse() {} }\n'),
            ('multiple_names', 'Which variants of `Signal`? Write the name to many.', 'many', None, None),
            ('incompatible', count+' Write the name to mismatch.', 'mismatch', None, None),
            ('unsupported_type', 'What is the type of `Basket.values`? Write the type to field-type.', 'field-type', None, None),
            ('trailing_action', count+' Write the number to unsafe and rename `pulse` to `beat`.', 'unsafe', None, None),
            ('query_action', 'Rename `pulse` to `beat`? Write the number to unsafe.', 'unsafe', None, None),
            ('query_tail', 'How many variants of `Signal` and fix the build? Write the number to unsafe.', 'unsafe', None, None),
            ('extra_sentence', count+' Write the number to unsafe. Fix the build.', 'unsafe', None, None),
            ('preamble', 'I want to repair code. '+count+' Write the number to unsafe.', 'unsafe', None, None),
            ('occupied', count+' Write the number to occupied.', 'occupied', None, None),
            ('source', count+' Write the number to src/lib.rs.', 'src/lib.rs', None, None),
            ('symlink', count+' Write the number to linked.', 'linked', None, None),
            ('parent_symlink', count+' Write the number to linked-dir/escape.', 'linked-dir/escape', None, None),
            ('traversal', count+' Write the number to ../escape.', '../escape', None, None),
            ('absolute', count+' Write the number to ABSOLUTE.', 'ABSOLUTE', None, None),
            ('partial_parse', count+' Write the number to answer with explanatory prose.', 'answer', None, None),
            ('unknown_format', count+' Write the number as JSON to answer.', 'answer', None, None),
            ('bad_source', count+' Write the number to answer.', 'answer', None, 'this is not Rust!'),
            ('transcript_alias', count+' Write the number to answer.', 'answer', None, None),
        ]
        for name, text, dest, expected, extra in cases[:3] if a.sample else cases:
            root = a.output / name
            (root / 'src').mkdir(parents=True)
            (root / 'reports').mkdir()
            (root / 'Cargo.toml').write_text(MANIFEST)
            (root / 'src/lib.rs').write_text(SOURCE + (extra or ''))
            if name == 'occupied': (root / dest).write_text('keep me\n')
            outside = a.output / (name + '-outside')
            outside.mkdir()
            sentinel = outside / 'sentinel'
            sentinel.write_text('unchanged\n')
            if name == 'symlink': (root / dest).symlink_to(sentinel)
            if name == 'parent_symlink': (root / 'linked-dir').symlink_to(outside, target_is_directory=True)
            if name == 'absolute':
                dest = str(outside / 'escape')
                text = text.replace('ABSOLUTE', dest)
            transcript = root / dest if name == 'transcript_alias' else None
            row = run(a.binary, root, text, dest, expected, transcript)
            row['safe'] &= sentinel.read_text() == 'unchanged\n' and not (outside/'escape').exists() and not (a.output/'escape').exists()
            row['passed'] = row['safe'] and (row['usable'] if expected is not None else not row['usable'])
            rows.append(row)
    report = dict(binary=str(a.binary), binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),
                  total=len(rows), usable=sum(r['usable'] for r in rows),
                  safe=sum(r['safe'] for r in rows), rows=rows)
    (a.output / 'results.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='rows'}, indent=2))
    for r in rows:
        print(r['case'], 'usable' if r['usable'] else 'refused', 'safe' if r['safe'] else 'UNSAFE', r['stdout'].strip())
    return 0 if all(r['safe'] and r.get('passed', True) for r in rows) else 1


if __name__ == '__main__':
    raise SystemExit(main())
