#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original authored CLI contracts; uses only the adjacent public fixtures."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
CASES = [
    ('struct', 'bounded', 'rename struct Parcel to Package', 'success'),
    ('broken', 'broken', 'rename struct Parcel to Package', 'failure'),
    ('disabled', 'bounded', 'rename struct Parcel to Package', 'disabled'),
    ('preview', 'bounded', 'rename struct Parcel to Package', 'preview'),
    ('function', 'bounded', 'rename function weigh to measure', 'success'),
    ('field', 'valid', 'rename field weight to mass', 'success'),
    ('multifile_rollback', 'multifile', 'rename struct Parcel to Package', 'failure'),
    ('session', 'bounded', 'rename struct Parcel to Package', 'session'),
    ('failed_session', 'broken', 'rename struct Parcel to Package', 'failed_session'),
    ('plain', 'bounded', 'rename struct Parcel to Package', 'plain'),
    ('explain', 'bounded', 'rename struct Parcel to Package', 'explain'),
    ('plain_preview', 'bounded', 'rename struct Parcel to Package', 'plain_preview'),
    ('query', 'bounded', 'how many structs are there', 'query'),
    ('declared_type', 'bounded', 'what is the type of field weight of struct Parcel', 'query'),
    ('unsupported_macro', 'valid', 'rename function weigh to measure', 'refusal'),
    ('unsupported_cfg', 'valid', 'rename struct Parcel to Package', 'refusal'),
]

def sha(data):
    return hashlib.sha256(data).hexdigest()

def files(root):
    return {str(p.relative_to(root)): sha(p.read_bytes()) for p in sorted(root.rglob('*'))
            if p.is_file() and (p.suffix == '.rs' or p.name == 'Cargo.toml')}

def invoke(binary, root, text, flags):
    cmd = [str(binary), 'ask', '-s', f'public={root}', *flags, text]
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    try:
        parsed = json.loads(p.stdout)
    except json.JSONDecodeError:
        parsed = None
    return {'command': cmd, 'exit': p.returncode, 'stdout': p.stdout,
            'stderr': p.stderr, 'json': parsed}

def run(binary, out, case):
    name, fixture, request, kind = case
    root = out / name / 'package'
    shutil.copytree(HERE / 'fixtures' / fixture, root)
    before = files(root)
    flags = [] if kind in ('plain', 'explain', 'plain_preview') else ['--json']
    if kind == 'explain':
        flags += ['--explain']
    if kind not in ('preview', 'plain_preview'):
        flags += ['--apply']
    if kind == 'disabled':
        flags += ['--disable', 'rust.apply.changes']
    session = root.parent / 'session.json'
    if kind in ('session', 'failed_session'):
        flags += ['--session', str(session)]
    r = invoke(binary, root, request, flags)
    r.update(name=name, before=before, after=files(root))
    checks = {}
    parsed = r['json'] or {}
    response = parsed.get('response', {})
    if kind in ('success', 'session'):
        checks['exit_success'] = r['exit'] == 0
        checks['source_changed'] = before != r['after']
        checks['single_checked_json'] = response.get('type') == 'checked' and response.get('passed') is True
        checks['verification_rule'] = 'rust.apply.changes' in parsed.get('rules', [])
    elif kind in ('failure', 'disabled', 'failed_session'):
        checks['exit_failure'] = r['exit'] != 0
        checks['source_unchanged'] = before == r['after']
        checks['single_refusal_json'] = response.get('type') == 'refuse'
        checks['no_success_claim'] = 'Applied' not in r['stdout'] and 'applied' not in parsed.get('text', '').lower()
    elif kind in ('preview', 'plain_preview'):
        checks['exit_success'] = r['exit'] == 0
        checks['source_unchanged'] = before == r['after']
        checks['proposal'] = response.get('type') == 'edit' if kind == 'preview' else "I'd rename" in r['stdout']
    elif kind in ('plain', 'explain'):
        checks['exit_success'] = r['exit'] == 0
        checks['source_changed'] = before != r['after']
        checks['final_verified_text'] = 'build passed' in r['stdout'].lower()
        if kind == 'explain':
            checks['trace_final_applied'] = 'reply:  Applied' in r['stdout']
            checks['trace_apply_rule'] = 'rust.apply.changes' in r['stdout']
    else:
        checks['exit_success'] = r['exit'] == 0
        checks['source_unchanged'] = before == r['after']
        checks['single_json'] = r['json'] is not None
        if kind == 'refusal':
            checks['unsupported'] = response.get('type') == 'refuse'
    if kind in ('session', 'failed_session'):
        saved = json.loads(session.read_text()) if session.exists() else None
        r['saved_session'] = saved
        if kind == 'session':
            checks['session_applied'] = saved is not None and 'Applied' in saved['turns'][-1]['reply']
            checks['one_record'] = saved is not None and len(saved['turns']) == 1
            follow = invoke(binary, root, 'yes', ['--json', '--session', str(session)])
            r['followup'] = follow
            checks['nothing_pending'] = (follow['json'] or {}).get('response', {}).get('reason') == 'nothing_pending'
            checks['followup_no_write'] = files(root) == r['after']
        else:
            checks['no_saved_success'] = saved is None or 'Applied' not in str(saved['turns'][-1]['reply'])
    r['checks'] = checks
    r['passed'] = all(checks.values())
    return r

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--binary', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--sample', action='store_true')
    ap.add_argument('--compare', type=Path)
    args = ap.parse_args()
    assert os.environ.get('TMPDIR') and not os.environ['TMPDIR'].startswith('/tmp')
    assert os.environ.get('CARGO_TARGET_DIR')
    args.out.mkdir(parents=True, exist_ok=False)
    result = {'binary': str(args.binary.resolve()), 'binary_sha256': sha(args.binary.read_bytes()),
              'fixture_hashes': files(HERE / 'fixtures'), 'results': []}
    for case in CASES[:4] if args.sample else CASES:
        r = run(args.binary.resolve(), args.out.resolve(), case)
        result['results'].append(r)
        print(case[0], 'PASS' if r['passed'] else 'FAIL', [k for k,v in r['checks'].items() if not v], flush=True)
    result['passed'] = sum(r['passed'] for r in result['results'])
    result['total'] = len(result['results'])
    if args.compare:
        old = {r['name']:r for r in json.loads(args.compare.read_text())['results']}
        result['byte_compatibility'] = {r['name']:r['stdout'] == old[r['name']]['stdout']
            for r in result['results'] if r['name'] in ('preview', 'plain_preview', 'query', 'declared_type', 'unsupported_macro', 'unsupported_cfg')}
    (args.out / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
    print(f"{result['passed']}/{result['total']} contracts passed")

if __name__ == '__main__':
    main()
