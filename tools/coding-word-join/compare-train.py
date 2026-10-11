"""Compare fixed TRAIN source outputs; verifier/results do not generate code."""
import hashlib,json,sys
from pathlib import Path
root=Path(sys.argv[1]).resolve()
baseline=Path('tools/coding-implementation-grammar/correction/train-full-v2/results.json')
old=json.loads(baseline.read_text());new=json.loads((root/'train-full/results.json').read_text())
a={r['id']:r for r in old};b={r['id']:r for r in new}
assert len(a)==len(b)==len(old)==len(new)==60 and a.keys()==b.keys()
retained=[];gained=[]
for key,row in a.items():
    assert row['before']==b[key]['before'],key
    if row['accepted']:
        assert b[key]['accepted'] and row['after']==b[key]['after'],key
        retained.append(key)
    elif b[key]['accepted']:gained.append(key)
    else:assert row['after']==b[key]['after'],key
verification=json.loads((root/'train-full-verification/results.json').read_text())
assert {r['id'] for r in verification}=={r['id'] for r in new if r['accepted']}
assert all(r['ok'] for r in verification)
report=dict(total=60,retained=retained,gained=gained,accepted=sum(r['accepted'] for r in new),
            original_result_sha256=hashlib.sha256(baseline.read_bytes()).hexdigest(),
            candidate_result_sha256=hashlib.sha256((root/'train-full/results.json').read_bytes()).hexdigest())
assert len(retained)==3 and len(gained)>=2
(root/'train-comparison.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
