from pathlib import Path
from fractions import Fraction
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();base=Path(__file__).parent
raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];cases=json.loads(raw);a.out.mkdir(parents=True,exist_ok=False)
inputs=''.join(json.dumps(c['question'])+'\n' for c in cases);(a.out/'inputs.jsonl').write_text(inputs)
r=subprocess.run([str(a.binary.resolve()),'--jsonl'],input=inputs,text=True,capture_output=True,timeout=120);(a.out/'predictions.jsonl').write_text(r.stdout);(a.out/'stderr.log').write_text(r.stderr);assert r.returncode==0
pred=[json.loads(s) for s in r.stdout.splitlines()];assert len(pred)==len(cases)
units={'second':1,'seconds':1,'s':1,'minute':60,'minutes':60,'min':60,'hour':3600,'hours':3600,'h':3600,'day':86400,'days':86400}
rows=[]
for c,v in zip(cases,pred):
 answered=v['status']=='answer';correct=False
 if answered and c['expected'] is not None and v.get('unit') in units:
  correct=Fraction(v['value'])*units[v['unit']]==Fraction(c['expected'])*units[c['unit']]
 rows.append(dict(name=c['name'],expected=c['expected'],expected_unit=c['unit'],actual=v,answered=answered,correct=correct,unsafe=answered and not correct))
result=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows,positive_correct=sum(x['correct'] for x in rows),wrong_answers=sum(x['unsafe'] for x in rows),negative_refusals=sum(c['expected'] is None and not x['answered'] for c,x in zip(cases,rows)))
(a.out/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='cases'},indent=2))
