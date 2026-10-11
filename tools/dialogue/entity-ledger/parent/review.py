from pathlib import Path
import argparse
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--model', type=Path, required=True)
p.add_argument('--schema', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
for name in ['freeze.json', 'baseline-freeze.json']:
    for file, expected in json.loads((HERE/name).read_text()).items():
        assert hashlib.sha256((HERE/file).read_bytes()).hexdigest() == expected, file
cases = json.loads((HERE/'cases.json').read_text())
baseline = json.loads((HERE/'baseline.json').read_text())
a.out.mkdir(parents=True, exist_ok=False)
pred = a.out/'predictions.json'
subprocess.run(['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G',
    '-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120',
    str(a.binary.resolve()),'predict','--model',str(a.model.resolve()),
    '--schema',str(a.schema.resolve()),'--input',str(HERE/'inputs.json'),
    '--output',str(pred.resolve())], check=True, timeout=150)
candidate = json.loads(pred.read_text())
rows = []
for case, old, new in zip(cases, baseline, candidate, strict=True):
    checks = []
    for before, after in zip(old, new, strict=True):
        checks.append(before['turn_index'] == after['turn_index'])
        checks.append(before['active_services'] == after['active_services'])
        old_frames = {f['service']:f['state'] for f in before['frames']}
        new_frames = {f['service']:f['state'] for f in after['frames']}
        checks.append(old_frames.keys() == new_frames.keys())
        for service, state in old_frames.items():
            got = new_frames[service]
            checks.append(state['active_intent'] == got['active_intent'])
            left, right = dict(state['slot_values']), dict(got['slot_values'])
            if service == 'RideSharing_1':
                was, now = left.pop('destination', None), right.pop('destination', None)
                if case['kind'] == 'control':
                    checks.append(now == was)
                else:
                    checks.append(now is None or now == [case['expected']])
                    if was == [case['expected']]:
                        checks.append(now == was)
            checks.append(left == right)
    final = next(f['state']['slot_values'].get('destination') for f in new[-1]['frames']
                 if f['service'] == 'RideSharing_1')
    rows.append(dict(name=case['name'], kind=case['kind'], delayed=case.get('delayed',False),
                     invariants=all(checks), destination=final,
                     correct=case['kind']=='positive' and final==[case['expected']]))
delayed = sum(r['delayed'] and r['correct'] for r in rows)
report = dict(cases=rows, delayed_correct=delayed, total_correct=sum(r['correct'] for r in rows),
              accepted=delayed>=2 and all(r['invariants'] for r in rows),
              provenance_review='Pending manual source-span audit; not established by this scorer',
              hashes={str(f):hashlib.sha256(f.read_bytes()).hexdigest()
                      for f in [a.binary,a.model,a.schema,pred]})
(a.out/'report.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['cases','hashes']},indent=2))
raise SystemExit(0 if report['accepted'] else 1)
