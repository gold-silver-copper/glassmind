#!/usr/bin/env python3
"""Independent ordinary-command and explanation regressions for query exports."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root = a.output / 'source'
(root / 'src').mkdir(parents=True)
(root / 'Cargo.toml').write_text('[package]\nname="export_regression"\nversion="0.1.0"\nedition="2024"\n')
source = root / 'src/lib.rs'
source.write_text('pub fn write() {}\npub fn put() {}\npub fn save() {}\npub fn caller() { write(); put(); save(); }\n')
source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
rows = []
for name in ['write', 'put', 'save']:
    for quoted in [False, True]:
        reference = f'`{name}`' if quoted else name
        text = f'How many functions call {reference}?'
        transcript = a.output / f'{name}-{quoted}.txt'
        result = subprocess.run([str(a.binary), 'do', str(root), '--reply-file', str(transcript), text], capture_output=True, text=True, timeout=30)
        expected = f'1 function calls `{name}`.\n'
        rows.append(dict(question=text, passed=result.returncode == 0 and result.stdout == expected and transcript.read_text() == expected, stdout=result.stdout))
for explain in [False, True]:
    name = f'count-{explain}.txt'
    # Deliberately omit whitespace after ?: it remains a clause boundary.
    text = f'How many functions call save?Write the number to {name}.'
    transcript = a.output / f'export-{explain}.txt'
    args = [str(a.binary), 'do', str(root), '--reply-file', str(transcript)]
    if explain:
        args.append('--explain')
    result = subprocess.run(args + [text], capture_output=True, text=True, timeout=30)
    answer = (root / name).read_text() if (root / name).exists() else None
    trace = all(part in result.stdout for part in ['Answer export:', 'Metadata question:', 'Resolved answer:', 'World rule:'])
    expected_message = f'Wrote 1 to {name}.\n'
    rows.append(dict(question=text, explain=explain, passed=result.returncode == 0 and answer == '1\n' and transcript.read_text() == expected_message and (trace if explain else result.stdout == expected_message), stdout=result.stdout))
assert hashlib.sha256(source.read_bytes()).hexdigest() == source_hash
report = dict(total=len(rows), right=sum(r['passed'] for r in rows), rows=rows,
              binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), source_sha256=source_hash)
(a.output / 'results.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k != 'rows'}))
assert report['right'] == report['total']
