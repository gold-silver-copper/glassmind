from pathlib import Path
import argparse
import hashlib
import json
import os
import subprocess

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--binary', required=True, type=Path)
parser.add_argument('--out', required=True, type=Path)
args = parser.parse_args()
binary = args.binary.resolve()
out = args.out.resolve()
out.mkdir(parents=True, exist_ok=False)
freeze = json.loads((HERE / 'boundary-freeze.json').read_text())
for name, expected in freeze.items():
    assert hashlib.sha256((HERE / name).read_bytes()).hexdigest() == expected, name
cases = json.loads((HERE / 'boundary-cases.json').read_text())
baseline = {row['name']: row for row in json.loads((HERE / 'baseline.json').read_text())}
rows = []
for case in cases:
    project = out / case['name']
    (project / 'src').mkdir(parents=True)
    (project / 'Cargo.toml').write_text('[package]\nname="route_probe"\nversion="0.1.0"\nedition="2021"\n')
    (project / 'src/lib.rs').write_text(case['source'])
    env = dict(os.environ)
    env.update(GLASSMIND_SCRATCH=str(out / 'scratch'), CARGO_TARGET_DIR=str(out / 'target'))
    result = subprocess.run([str(binary), 'do', str(project), case['request']], env=env,
                            text=True, capture_output=True, timeout=90)
    after = {str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in project.rglob('*') if p.is_file()}
    old = baseline[case['name']]
    # Preserve exact outputs and every project byte observed in the old route.
    checks = dict(exit=result.returncode == old['exit'], stdout=result.stdout == old['stdout'],
                  stderr=result.stderr == old['stderr'], files=after == old['after'])
    rows.append(dict(name=case['name'], ok=all(checks.values()), checks=checks,
                     exit=result.returncode, stdout=result.stdout, stderr=result.stderr, after=after))
report = dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
              total=len(rows), passed=sum(row['ok'] for row in rows), cases=rows)
(out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: v for k, v in report.items() if k != 'cases'}))
raise SystemExit(0 if report['passed'] == report['total'] else 1)
