"""Exercise task persistence through separate real CLI processes."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

binary, out = map(lambda s: Path(s).resolve(), sys.argv[1:])
out.mkdir()
root = out / 'project'
(root / 'src').mkdir(parents=True)
(root / 'Cargo.toml').write_text('[package]\nname="task_cli_smoke"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
original = 'pub struct Sensor;\npub fn measure(input: Sensor) -> Sensor { input }\n'
source = root / 'src/lib.rs'
source.write_text(original)
session = out / 'session.json'
env = dict(os.environ, GLASSMIND_SCRATCH=str(out / 'scratch'),
           GLASSMIND_CARGO_TARGET=str(out / 'cargo-target'),
           GLASSMIND_CHECK_CACHE=str(out / 'check-cache'),
           CARGO_TARGET_DIR=str(out / 'cargo-target'))
turns = []

def turn(text, steps=2):
    p = subprocess.run([str(binary), 'do', str(root), '--session', str(session),
                        '--steps', str(steps), text], env=env, capture_output=True,
                       text=True, timeout=180)
    turns.append(dict(request=text, code=p.returncode, stdout=p.stdout, stderr=p.stderr))
    (out / 'turns.json').write_text(json.dumps(turns, indent=2) + '\n')
    assert p.returncode == 0, turns[-1]
    return json.loads(session.read_text())

state = turn('Could you rename the Sensor struct to Probe, please?', 1)
assert state['task']['status'] == 'Ready' and source.read_text() == original
state = turn('continue')
assert state['task']['status'] == 'Completed'
assert source.read_text() == original.replace('Sensor', 'Probe')
state = turn('rename it to Detector')
assert state['task']['status'] == 'Completed'
assert source.read_text() == original.replace('Sensor', 'Detector')
turn('undo that change')
assert source.read_text() == original.replace('Sensor', 'Probe')
turn('rename Probe to Device', 1)
external = source.read_text() + '// external change must survive\n'
source.write_text(external)
state = turn('continue')
assert state['task']['status'] == 'Stale'
assert state['conversation']['turns'] == [] and source.read_text() == external
turn('show task')
# Ordered actions survive real process boundaries and preserve the external edit.
state = turn('rename Probe to Meter; then rename it to Gauge; then check the tests', 1)
assert state['plan']['status'] == 'Ready' and state['plan']['current'] == 0
assert source.read_text() == external
state = turn('continue', 1)
assert state['plan']['current'] == 1
assert source.read_text() == external.replace('Probe', 'Meter')
state = turn('continue', 8)
assert state['plan']['status'] == 'Completed' and state['plan']['current'] == 3
assert source.read_text() == external.replace('Probe', 'Gauge')
turn('undo')
assert source.read_text() == external.replace('Probe', 'Meter')
turn('continue', 8)
assert source.read_text() == external.replace('Probe', 'Meter')
state = turn('rename Meter to Reader; then delete every file', 8)
assert source.read_text() == external.replace('Probe', 'Meter')
assert state['plan']['status'] == 'Cancelled'
# Ambiguous owner choices persist, keep the new name, and update follow-up context.
root = out / 'choice-project'
(root / 'src').mkdir(parents=True)
(root / 'Cargo.toml').write_text('[package]\nname="choice_cli_smoke"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
source = root / 'src/lib.rs'
original_choice = 'pub struct North { pub value: u32 }\npub struct South { pub value: u32 }\npub fn south(s: South) -> u32 { s.value }\n'
source.write_text(original_choice)
session = out / 'choice-session.json'
state = turn('rename field value to count; then check the tests', 8)
assert state['plan']['status'] == 'Blocked' and source.read_text() == original_choice
assert len(state['task']['choices']['candidates']) == 2
state = turn('choose North::value', 1)
assert state['task']['status'] == 'Ready' and source.read_text() == original_choice
state = turn('use the other one', 1)
assert state['task']['choices']['selected'] == 1
state = turn('continue', 8)
assert state['plan']['status'] == 'Completed'
assert 'South { pub count: u32 }' in source.read_text()
assert 'North { pub value: u32 }' in source.read_text()
state = turn('rename it to amount', 8)
assert state['task']['status'] == 'Completed'
assert 'South { pub amount: u32 }' in source.read_text()
assert 's.amount' in source.read_text()
turn('undo')
assert 'South { pub count: u32 }' in source.read_text()
assert 'North { pub value: u32 }' in source.read_text()
# A compiler fix exposes a test failure; intermediate repairs remain isolated.
root = out / 'revision-project'
(root / 'src').mkdir(parents=True)
(root / 'Cargo.toml').write_text('[package]\nname="revision_cli_smoke"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
source = root / 'src/lib.rs'
original_revision = 'pub fn combine(a: i32, b: i32) -> i32 { let answer = a - b; answer += 0; answer }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(combine(9,4),13); assert_eq!(combine(-6,2),-4); } }\n'
source.write_text(original_revision)
session = out / 'revision-session.json'
state = turn('fix the tests', 1)
assert state['task']['status'] == 'Revising' and source.read_text() == original_revision
state = turn('continue', 1)
assert len(state['task']['revision']['attempts']) == 1
assert state['task']['status'] == 'Revising' and source.read_text() == original_revision
state = turn('continue', 1)
assert state['task']['status'] == 'Ready' and source.read_text() == original_revision
state = turn('continue', 1)
assert state['task']['status'] == 'Completed'
assert len(state['task']['revision']['attempts']) == 2
assert 'let mut answer' in source.read_text() and 'a + b' in source.read_text()
turn('undo')
assert source.read_text() == original_revision
report = dict(processes=len(turns), passed=True,
              binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
(out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
