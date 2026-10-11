#!/usr/bin/env python3
import argparse, pathlib, json, hashlib
p=argparse.ArgumentParser();p.add_argument('--source',required=True,type=pathlib.Path);p.add_argument('--artifacts',required=True,type=pathlib.Path);a=p.parse_args()
a.artifacts.mkdir(parents=True,exist_ok=True);ds=json.loads((a.source/'train.json').read_text())
(a.artifacts/'train-input.json').write_text(json.dumps([[{k:t[k] for k in ('speaker','utterance')} for t in d['turns']] for d in ds]))
(a.artifacts/'split.json').write_text(json.dumps({d['dialogue_id']:('CAL' if int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0 else 'FIT') for d in ds}))
