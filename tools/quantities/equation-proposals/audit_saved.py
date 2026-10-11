#!/usr/bin/env python3
"""Verify saved evidence and count calibration representation coverage; no inference."""
import hashlib,json,os,sys
from pathlib import Path
import pyarrow.parquet as pq
root=Path(__file__).resolve().parents[3];run=Path(os.environ['PROPOSAL_RUN']);source=Path(os.environ['PROPOSAL_SOURCE'])
sys.path.insert(0,str(root/'tools/quantities/typed-label-metric'))
from metric import parse_gold,compare
for name in ['prefit-manifest.json','calibration-prescore-manifest.json','sample-prescore-manifest.json','full-prescore-manifest.json','final-manifest.json']:
    for path,item in json.loads((run/name).read_text()).items():
        p=Path(path);assert p.stat().st_size==item['bytes'],p
        assert hashlib.sha256(p.read_bytes()).hexdigest()==item['sha256'],p
split=json.loads((run/'split.json').read_text());model=json.loads((run/'model.json').read_text())
key=lambda p:json.dumps(p,sort_keys=True,separators=(',',':'))
bank={key(c['program']) for c in model['classes']}
executions=[json.loads(l) for l in (run/'train-executions.jsonl').read_text().splitlines()]
labels=pq.read_table(source/'train.parquet',columns=['options','correct']).to_pylist()
counts={}
for part in ['fit','calibration']:
    indices=split['fit' if part=='fit' else 'calibration']
    executable_bank=clean_bank=0
    for i in indices:
        o=executions[i]
        if o['status']=='proposed' and key(o['program']) in bank:
            executable_bank+=1
            _,gold=parse_gold(labels[i]['options'],labels[i]['correct'])
            clean_bank+=compare(gold,{'status':'answer','value':o['value'],'unit':None})['grade']=='correct'
    counts[part]={'all_rows':len(indices),'executable_annotation_in_candidate_bank':executable_bank,'exact_label_agreeing_annotation_in_candidate_bank':clean_bank}
counts['purpose']='Post-fit descriptive representation audit only; all calibration rows were predicted and scored; no tuning.'
(run/'eligibility-audit.json').write_text(json.dumps(counts,indent=2)+'\n')
print(json.dumps(counts))
