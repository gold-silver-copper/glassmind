#!/usr/bin/env python3
"""Exact legacy bytes, question-only runtime, first200 then full development."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument('--binary',required=True);p.add_argument('--predictor',required=True);p.add_argument('--bundle',type=Path,required=True);p.add_argument('--valid',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--baseline',type=Path);a=p.parse_args()
assert a.valid.name=='valid.tsv';a.output.mkdir(parents=True,exist_ok=True)
rows=[r.split('\t',3) for r in a.valid.read_text().splitlines()];assert len(rows)==2821
reports=[]
for count in [200,2821]:
 q=[r[3] for r in rows[:count]]; inputs=''.join('/reset\n'+x+'\n' for x in q)
 for name,cmd,text in [('standalone',[a.predictor,a.bundle/'relation.json',a.bundle/'index.json'],'\n'.join(q)+'\n'),('talk',[a.binary,'talk','--json','--facts-dir',a.bundle],inputs),('default',[a.binary,'talk','--json'],inputs)]:
  raw=subprocess.run(list(map(str,cmd)),input=text.encode(),capture_output=True,timeout=550,check=True).stdout
  file=f'{name}-{count}.jsonl';(a.output/file).write_bytes(raw)
  if a.baseline:assert raw==(a.baseline/file).read_bytes(),file
  reports.append(dict(mode=name,rows=count,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),identical=bool(a.baseline)))
(a.output/'report.json').write_text(json.dumps(reports,indent=2)+'\n');print(json.dumps(reports))
