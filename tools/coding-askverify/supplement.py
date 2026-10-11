#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Additional public contracts; definitions frozen before running either binary."""
import argparse
import json
from pathlib import Path
import shutil
from run import HERE, files, invoke, sha
import subprocess

ap = argparse.ArgumentParser()
ap.add_argument('--binary', type=Path, required=True)
ap.add_argument('--out', type=Path, required=True)
args = ap.parse_args()
args.out.mkdir(parents=True, exist_ok=False)
binary = args.binary.resolve()
results = []

def fixture(name, source='bounded'):
    root = args.out.resolve() / name
    shutil.copytree(HERE / 'fixtures' / source, root)
    return root

def record(name, calls, checks):
    results.append(dict(name=name, calls=calls, checks=checks, passed=all(checks.values())))
    print(name, results[-1]['passed'], flush=True)

root = fixture('json_explain')
r = invoke(binary, root, 'rename struct Parcel to Package', ['--apply', '--json', '--explain'])
record('json_explain', [r], {'single_checked_json': (r['json'] or {}).get('response', {}).get('passed') is True,
                           'exit_success': r['exit'] == 0})

root = fixture('declared')
before = files(root)
request = 'what is the type of Parcel.weight'
a = invoke(binary, root, request, ['--json'])
b = invoke(binary, root, request, ['--json', '--apply'])
c = invoke(binary, root, request, ['--json', '--apply', '--session', str(root.parent / 'declared-session.json')])
record('declared_type_success_and_session_guard', [a,b,c], {
    'declared_u32': (b['json'] or {}).get('declared_type', {}).get('spelling') == 'u32',
    'apply_same_as_query': a['stdout'] == b['stdout'] and a['exit'] == b['exit'] == 0,
    'source_unchanged': files(root) == before,
    'session_guard': c['exit'] != 0 and not (root.parent / 'declared-session.json').exists()})

root = fixture('history')
session = root.parent / 'history-session.json'
flags = ['--json', '--session', str(session)]
a = invoke(binary, root, 'rename struct Parcel to Package', flags)
b = invoke(binary, root, 'rename struct Parcel to Package', flags + ['--apply'])
saved = json.loads(session.read_text())
before_yes = files(root)
c = invoke(binary, root, 'yes', flags)
record('preview_then_apply_history', [a,b,c, {'session_before_yes':saved}], {
    'proposal_then_applied': len(saved['turns']) == 2 and 'Proposal' in saved['turns'][0]['reply'] and 'Applied' in saved['turns'][1]['reply'],
    'nothing_pending': (c['json'] or {}).get('response', {}).get('reason') == 'nothing_pending',
    'yes_no_write': files(root) == before_yes})

root = fixture('selected')
other = fixture('other', 'routing')
before, before_other = files(root), files(other)
cmd = [str(binary), 'ask', '-s', f'selected={root}', '-s', f'other={other}', '--apply', '--json', 'rename function weigh to measure']
p = subprocess.run(cmd, text=True, capture_output=True, timeout=180)
try:
    parsed = json.loads(p.stdout)
except json.JSONDecodeError:
    parsed = {}
record('two_source_routing', [dict(command=cmd, exit=p.returncode, stdout=p.stdout, stderr=p.stderr)], {
    'selected_changed': files(root) != before,
    'other_unchanged': files(other) == before_other,
    'checked_success': p.returncode == 0 and parsed.get('response', {}).get('passed') is True})

out = dict(binary=str(binary), binary_sha256=sha(binary.read_bytes()), results=results,
           passed=sum(r['passed'] for r in results), total=len(results), fixture_hashes=files(HERE / 'fixtures'))
(args.out / 'results.json').write_text(json.dumps(out, indent=2) + '\n')
