"""Verify the integrated CLI against retained public source-name predictions."""
import hashlib
import json
from pathlib import Path
import subprocess

run = Path(__file__).resolve().parent
binary = Path('/home/gsc/code/glassmind/target/release/glassmind-facts')
model = run.parent.parent / 'trainindex-run/tmp/trainindex/bundle1000/relation.json'
destination = run / 'central-source-name-predictions.jsonl'
with (run / 'source-name-queries.txt').open('rb') as inp, destination.open('xb') as out:
    subprocess.run([str(binary), 'predict-disk', '--model', str(model), '--database', str(run / 'native.sqlite')],
                   stdin=inp, stdout=out, check=True, timeout=120)
expected = [json.loads(s) for s in (run / 'source-name-predictions.jsonl').read_text().splitlines()]
found = [json.loads(s) for s in destination.read_text().splitlines()]
assert len(found) == 853 and found == expected
report = {'rows': len(found), 'matched': len(found), 'artifacts': []}
for path in [binary, binary.parent / 'glassmind', destination]:
    with path.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    report['artifacts'].append({'path': str(path), 'bytes': path.stat().st_size, 'sha256': digest})
(run / 'central-query-report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report), flush=True)
