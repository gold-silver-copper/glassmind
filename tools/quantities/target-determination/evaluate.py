"""Fixed MathQA DEV, existing frozen v3 metric, question-only native inference."""
from pathlib import Path
import argparse, collections, hashlib, json, subprocess, sys
HERE = Path(__file__).resolve().parent
ROOT = Path('/home/gsc/code/glassmind')
METRIC = ROOT/'tools/quantities/typed-label-metric'
sys.path.insert(0, str(METRIC))
from metric import parse_gold, compare
from replay import PARQUET, QHASH
import pyarrow.parquet as pq

sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
p = argparse.ArgumentParser()
p.add_argument('--sample', action='store_true')
a = p.parse_args()
out = HERE/('sample' if a.sample else 'full')
out.mkdir(exist_ok=False)
for name, h in json.loads((METRIC/'FREEZE.json').read_text())['files'].items():
    assert sha(METRIC/name) == h
data = HERE.parents[1]/'quantities-run/tmp/mathqa'
for split, (h, size) in PARQUET.items():
    assert sha(data/(split+'.parquet')) == h
    assert (data/(split+'.parquet')).stat().st_size == size
questions = pq.read_table(data/'validation.parquet', columns=['Problem']).column(0).to_pylist()
assert len(questions) == 4475
assert hashlib.sha256(''.join(json.dumps(q)+'\n' for q in questions).encode()).hexdigest() == QHASH
qs = questions[:100] if a.sample else questions
stream = ''.join(json.dumps(q)+'\n' for q in qs)
(out/'questions.jsonl').write_text(stream)
pred = {}
for name in ['baseline','candidate']:
    binary = HERE/name
    r = subprocess.run([str(binary),'--jsonl'], input=stream, text=True, capture_output=True, check=True, timeout=180)
    (out/(name+'.jsonl')).write_text(r.stdout)
    pred[name] = [json.loads(x) for x in r.stdout.splitlines()]
    assert len(pred[name]) == len(qs)
(out/'inference.json').write_text(json.dumps({name:dict(binary_sha256=sha(HERE/name),predictions_sha256=sha(out/(name+'.jsonl'))) for name in pred},indent=2)+'\n')
goldrows = pq.read_table(data/'validation.parquet',columns=['options','correct']).to_pylist()[:len(qs)]
scores = {name:collections.Counter() for name in pred}
changes, lost, new_bad = [], [], []
retained = 0
for i, (question,row) in enumerate(zip(qs,goldrows,strict=True)):
    _, gold = parse_gold(row['options'],row['correct'])
    before, after = pred['baseline'][i], pred['candidate'][i]
    grades = {name:compare(gold,pred[name][i])['grade'] for name in pred}
    for name,grade in grades.items(): scores[name][grade] += 1
    if before['status']=='answer':
        if before == after: retained += 1
        else: lost.append(i)
    if after['status']=='answer' and after != before:
        changes.append(dict(index=i,question=question,before=before,after=after,grades=grades))
        if grades['candidate'] in ['wrong','unscorable']: new_bad.append(i)
summary = dict(total=len(qs),scores=scores,retained_answers=retained,changed_old_answers=lost,new_bad=new_bad,new_answers=changes,gate=not lost and not new_bad)
(out/'report.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
assert summary['gate']
