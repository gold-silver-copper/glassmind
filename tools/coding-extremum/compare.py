import argparse,json,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);a=p.parse_args();r=a.run
base=json.loads((r/'baseline-full/report.json').read_text());new=json.loads((r/'candidate-full/report.json').read_text())
assert base['inputs_sha256']==new['inputs_sha256']=='6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164'
assert base['totals']=={'verified':7,'refused':20} or base['totals']=={'refused':20,'verified':7}
retained=[];gains=[]
for i,(b,n) in enumerate(zip(base['details'],new['details'],strict=True)):
 assert b['source']==n['source']
 if b['verdict']=='verified':
  assert b==n,(b,n)
  old=json.loads((r/f'baseline-full/{i}-outcome.json').read_text());now=json.loads((r/f'candidate-full/{i}-outcome.json').read_text())
  assert old['response']==now['response'],b['source']
  retained.append(b['source'])
 elif n['verdict']=='verified':gains.append(n)
assert len(retained)==7 and len(gains)>=1
assert not new['totals'].get('failed_verification',0)
report=dict(baseline=base['totals'],candidate=new['totals'],retained_exact_responses=retained,gains=gains,selection_sha256=new['inputs_sha256'])
(r/'comparison.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
