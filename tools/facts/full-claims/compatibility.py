"""Regressions for the new reader/importer; uses the unchanged format-1 graph."""
import hashlib
import json
from pathlib import Path
import subprocess

RUN = Path(__file__).resolve().parent
OLD = RUN.parent / 'disk-facts'
WORK = RUN.parent.parent
REPO = WORK / 'full-claims'
BIN = OLD / 'target/release'
OUT = RUN / 'compatibility'
OUT.mkdir()

def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


checks = []
for entry in json.loads((OLD / 'inputs.json').read_text()):
    path = Path(entry['path'])
    assert path.stat().st_size == entry['bytes'] and digest(path) == entry['sha256']
assert digest(OLD / 'native999.sqlite') == '1c0450201e9992bf7ee33f77f5a38039d045634efc020e44ff8323b7ac3958cd'
for split, rows in [('train', 19481), ('valid', 2821)]:
    for mode in ['answer', 'set']:
        expected = OLD / f'integrated-{split}-{mode}.jsonl'
        command = [str(BIN / 'glassmind-facts'), 'predict-disk', '--model', str(WORK / 'trainindex-run/tmp/trainindex/bundle1000/relation.json'),
                   '--database', str(OLD / 'native999.sqlite')]
        if mode == 'set':
            command.append('--set')
        with (WORK / f'linkquality-run/artifacts/{split}.questions').open('rb') as questions:
            result = subprocess.run(command, stdin=questions, capture_output=True, check=True, timeout=120)
        found = [json.loads(line) for line in result.stdout.splitlines()]
        assert len(found) == rows and found == [json.loads(line) for line in expected.read_text().splitlines()]
        destination = OUT / f'{split}-{mode}.jsonl'
        destination.write_bytes(result.stdout)
        checks.append({'split': split, 'mode': mode, 'matched': rows, 'expected_sha256': digest(expected), 'output_sha256': digest(destination)})
for mode in ['plain', 'intent']:
    command = ['python3', str(REPO / 'tools/facts/chat/mixed.py'), '--binary', str(BIN / 'glassmind'),
               '--facts-dir', str(OLD / 'legacy-main-bundle'), '--output', str(OUT / f'main-{mode}')]
    if mode == 'intent':
        command += ['--model', '/home/gsc/.cache/glassmind/models/everyday/intent.json']
    result = subprocess.run(command, cwd=REPO, capture_output=True, check=True, timeout=300)
    checks.append({'main_mode': mode, 'result': json.loads(result.stdout)})
public = RUN.parent / 'wikidata-native'
command = [str(BIN / 'glassmind-facts'), 'predict-disk', '--model', str(WORK / 'trainindex-run/tmp/trainindex/bundle1000/relation.json'), '--database', str(public / 'native.sqlite')]
with (public / 'source-name-queries.txt').open('rb') as questions:
    result = subprocess.run(command, stdin=questions, capture_output=True, check=True, timeout=120)
assert result.stdout == (public / 'source-name-predictions.jsonl').read_bytes()
checks.append({'format2_source_queries': 853, 'prediction_sha256': hashlib.sha256(result.stdout).hexdigest()})
result = subprocess.run([str(RUN / 'disk-witness-probe'), str(public / 'native.sqlite'), str(public / 'witness-cases.json')], capture_output=True, check=True, timeout=120)
checks.append({'format2_positive_witnesses': json.loads(result.stdout)})
report = {'checks': checks, 'facts_binary_sha256': digest(BIN / 'glassmind-facts'), 'main_binary_sha256': digest(BIN / 'glassmind')}
(OUT / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report), flush=True)
