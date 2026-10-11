#!/usr/bin/env python3
"""Compare unchanged authored adapters without redefining their outcomes."""
import hashlib,json,os,sys
from pathlib import Path
import evaluate
run=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
def read(p): return json.loads(p.read_text())
base=read(run/'baseline-full/results.json');candidate=read(run/'accepted-full/results.json')
assert len(base)==len(candidate)==16
assert [r['id'] for r in base]==[r['id'] for r in candidate]
assert all(r['pass'] for r in candidate)
assert all(not b['pass'] or c['pass'] for b,c in zip(base,candidate))
assert all(r['behavior'] for r in base+candidate)
assert all(r['pass'] and not r['changed'] for r in read(run/'accepted-audit/results.json'))
keys={'pass','passed','safe','usable','changed','exact_source','behavior','supported','preserved','structural','behavior_preserved','lock_preserved'}
def outcome(x):
 if isinstance(x,list): return [outcome(y) for y in x]
 if isinstance(x,dict): return {k:outcome(v) for k,v in x.items() if k in keys or k in ('rows','cases','results')}
 return x
regressions={}
for path in sorted((run/'regressions-baseline').rglob('results.json')):
 rel=path.relative_to(run/'regressions-baseline');b=read(path);c=read(run/'regressions-accepted'/rel)
 assert outcome(b)==outcome(c),str(rel)
 regressions[str(rel)]=outcome(c)
for rel in ['status.txt','addfield/status.txt']:
 assert (run/'regressions-baseline'/rel).read_text()==(run/'regressions-accepted'/rel).read_text()
summary=dict(base='5f2859a730f6d20a0ac64ca9a7551821ed123e86',baseline_correct=sum(r['pass'] for r in base),candidate_correct=sum(r['pass'] for r in candidate),total=len(base),new_exact_edits=sum(r['changed'] for r in candidate),supplemental_refusals=4,fixture_source_bytes=sum(len(c['source'].encode()) for c in evaluate.CASES),fixture_expected_bytes=sum(len(c['expected'].encode()) for c in evaluate.CASES),license='MIT OR Apache-2.0',download_bytes=0,regressions=regressions)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print({k:v for k,v in summary.items() if k!='regressions'})
