#!/usr/bin/env python3
"""Post-freeze adapter contract checks; not part of linguistic acceptance scoring."""
import json
import pathlib
import subprocess
import sys

out = pathlib.Path(sys.argv[1])
binary, model = out/'reading-syntax', out/'model.bin'
cases = [
    ({'forms': ['Birds', 'fly', '.']}, 'predicted'),
    ({'text': 'Élodie can\'t—go! e\u0301'}, 'predicted'),
    ({'forms': ['x'], 'heads': [0]}, 'refused'),
    ({'forms': ['x'], 'upos': ['NOUN']}, 'refused'),
    ({'forms': ['x'], 'id': 'gold'}, 'refused'),
    ({'forms': ['x'], 'lemmas': ['x']}, 'refused'),
    ({'forms': ['x'], 'deprels': ['root']}, 'refused'),
    ({'forms': ['x'], 'features': ['X']}, 'refused'),
    ({'forms': ['x'], 'text': 'x'}, 'refused'),
    ({'text': ''}, 'refused'),
    ({'forms': []}, 'refused'),
    ({'forms': ['x']*129}, 'refused'),
    ({'forms': ['x'*16385]}, 'refused'),
    ({'text': 'x'*16385}, 'refused'),
    ({'forms': ['two words']}, 'refused'),
    ({'forms': ['\x00']}, 'refused'),
]
input_path, output_path = out/'contracts-input.jsonl', out/'contracts-output.jsonl'
input_path.write_text(''.join(json.dumps(c)+'\n' for c, _ in cases))
command = [str(binary), 'predict', '--model', str(model), '--input', str(input_path), '--output', str(output_path)]
subprocess.run(command, check=True)
predictions = [json.loads(l) for l in output_path.read_text().splitlines()]
assert len(predictions) == len(cases)
for (c, expected), p in zip(cases, predictions):
    assert p['status'] == expected, (c, expected, p)
    if 'spans' in p and p['spans'] is not None:
        raw = c['text'].encode()
        assert [raw[t['start']:t['end']].decode() for t in p['spans']] == p['forms']
# Verify invalid sparse finite/nonfinite payload is rejected by actual loaded model.
raw = model.read_bytes()
corrupt = bytearray(raw)
corrupt[-4:] = (0x7fc00000).to_bytes(4, 'little')  # last weight: quiet NaN
bad = out/'corrupt-model.bin'
bad.write_bytes(corrupt)
command[command.index('--model')+1] = str(bad)
r = subprocess.run(command, capture_output=True, text=True)
assert r.returncode != 0 and 'invalid sparse weight' in r.stderr
(out/'contracts-report.json').write_text(json.dumps({'cases': len(cases), 'passed': len(cases),
    'actual_model_nan_rejected': True, 'error': r.stderr.strip(),
    'scope': 'input/serialization contracts, not linguistic accuracy'}, indent=2)+'\n')
print('16 input/span contracts passed; learned-model NaN rejected')
