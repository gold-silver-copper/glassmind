"""Compare existing authored development sets, with unchanged expectations."""
import argparse,json,subprocess
from pathlib import Path
from fractions import Fraction
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
def run(binary,rows):
 r=subprocess.run([str(binary),'--jsonl'],input=''.join(json.dumps(x['question'])+'\n' for x in rows),text=True,capture_output=True,check=True,timeout=120)
 a=[json.loads(x) for x in r.stdout.splitlines()];assert len(a)==len(rows);return a
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--baseline',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
sets={}
sets['synthetic']=[json.loads(l) for name in ['synthetic-dev.jsonl','extended-dev.jsonl'] for l in (ROOT/'crates/quantities/dev'/name).read_text().splitlines()]
sets['percentchange']=[json.loads(l) for l in (HERE.parent/'percent-change/probe-inputs.jsonl').read_text().splitlines()]
sets['parent']=json.loads((HERE.parent/'percent-change/parent/frozen-cases.json').read_text())
sets['authored']=[r for f in ['authored.json','supplemental.json','safety.json','ellipsis.json'] for r in json.loads((HERE/f).read_text())]
report={}
for name,rows in sets.items():
 before=run(a.baseline,rows);after=run(a.binary,rows);changes=[];right=refused=wrong=expected_refusals=0
 for i,(row,b,c) in enumerate(zip(rows,before,after)):
  expected=row.get('expected',row.get('value'));unit=row.get('unit')
  canonical=lambda x:(x['status'],x.get('value'),x.get('unit'))
  if canonical(b)!=canonical(c):changes.append(i)
  if expected is None:
   if c['status']=='refusal':right+=1;expected_refusals+=1
   else:wrong+=1
  elif c['status']=='refusal':refused+=1
  elif c['value']==expected and c.get('unit')==unit:right+=1
  else:wrong+=1
 report[name]=dict(total=len(rows),correct=right,refused=refused,wrong=wrong,expected_refusals=expected_refusals,changed_outcomes=changes)
 if name!='authored':assert not changes,(name,changes)
 assert not wrong,(name,wrong)
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
