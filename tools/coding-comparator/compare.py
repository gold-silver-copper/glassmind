"""Exact fixed public TRAIN comparison; never loads hidden assets."""
import argparse,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);r=p.parse_args().run
base=json.loads((r/'baseline-full/report.json').read_text());new=json.loads((r/'candidate-full/report.json').read_text())
assert base['inputs_sha256']==new['inputs_sha256']=='6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164'
assert base['totals']=={'verified':9,'refused':18}
assert len(base['details'])==len(new['details'])==27
retained=[];gains=[]
for i,(b,n) in enumerate(zip(base['details'],new['details'],strict=True)):
 assert b['source']==n['source']
 if b['verdict']=='verified':
  assert b==n
  assert (r/f'baseline-full/{i}-outcome.json').read_bytes()==(r/f'candidate-full/{i}-outcome.json').read_bytes(),b['source']
  retained.append(b['source'])
 elif n['verdict']=='verified':gains.append(n)
assert len(retained)==9 and len(gains)>=1
assert not new['totals'].get('failed_verification',0)
bs=json.loads((r/'baseline-sample/report.json').read_text());ns=json.loads((r/'candidate-sample/report.json').read_text())
assert bs['totals']=={'verified':2,'refused':2}
assert bs['inputs_sha256']==ns['inputs_sha256'] and len(ns['details'])==4
assert not ns['totals'].get('failed_verification',0)
report=dict(baseline=base['totals'],candidate=new['totals'],retained_entire_outcome_bytes=retained,gains=gains,selection_sha256=new['inputs_sha256'],baseline_sample=bs['totals'],candidate_sample=ns['totals'])
(r/'comparison.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
