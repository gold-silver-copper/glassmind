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
report = dict(processes=len(turns), passed=True,
              binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
(out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
