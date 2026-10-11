#!/usr/bin/env python3
"""Attach a value-free resolver without retraining or changing frozen weights."""
import argparse, json, pathlib
p=argparse.ArgumentParser()
for name in ('baseline','relations','output'):p.add_argument('--'+name,required=True,type=pathlib.Path)
a=p.parse_args()
model=json.loads(a.baseline.read_text())
assert 'references' not in model, 'expected frozen baseline without resolver'
model['references']=json.loads(a.relations.read_text())
a.output.write_text(json.dumps(model,separators=(',',':')))
check=json.loads(a.output.read_text());del check['references']
assert check==json.loads(a.baseline.read_text()), 'frozen weights changed'
