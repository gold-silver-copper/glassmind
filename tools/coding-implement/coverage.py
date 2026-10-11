#!/usr/bin/env python3
"""Inspect only TRAIN request syntax, never expected labels or source tests."""
import json, pathlib, re
root=pathlib.Path('/home/gsc/.cache/glassmind/data/train-v3')
allowed=set()
for line in (root/'labels.jsonl').open():
    row=json.loads(line)
    if row.get('split')=='train' and row.get('kind')=='rb_implement':
        allowed.add(row['conversation'])
rows=[]
for line in (root/'inputs.jsonl').open():
    row=json.loads(line)
    if row['id'] not in allowed: continue
    for turn in row['turns']:
        words=turn['text'].split()
        recognized=bool(words and words[0]=='implement')
        accepted=bool(recognized and len(words)==2 and re.fullmatch(r'[^\W\d]\w*|_\w*',words[1]))
        rows.append(dict(id=turn['id'],recognized=recognized,accepted_syntax=accepted))
report=dict(note='Syntax audit only, not benchmark execution or semantic scoring. No expected values inspected.',train_requests=len(rows),accepted_syntax=sum(r['accepted_syntax'] for r in rows),rows=rows)
print(json.dumps(report,indent=2))
