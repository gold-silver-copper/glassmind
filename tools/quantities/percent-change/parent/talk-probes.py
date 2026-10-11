from pathlib import Path
from fractions import Fraction
import subprocess,json
out=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/percentchange-review')
rows=json.loads((out/'frozen-cases.json').read_text())
stream=''.join('/reset\n'+r['question']+'\n' for r in rows)
reports=[]
for mode,args in [('plain',[]),('model',['--model','/home/gsc/.cache/glassmind/models/everyday/intent.json'])]:
 p=subprocess.run([str(out/'glassmind-integrated'),'talk','--json']+args,input=stream,text=True,capture_output=True,check=True,timeout=60)
 replies=[json.loads(x) for x in p.stdout.splitlines()];assert len(replies)==2*len(rows)
 results=[]
 for r,reset,reply in zip(rows,replies[::2],replies[1::2]):
  assert reset['kind']=='reset'
  ok=reply['kind']!='calculated' if r['value'] is None else reply['kind']=='calculated' and Fraction(reply['value'])==Fraction(r['value']) and reply['unit']==r['unit']
  results.append(dict(question=r['question'],passed=ok,reply=reply,expected=r['value']))
 report=dict(mode=mode,total=len(results),passed=sum(x['passed'] for x in results),positives=sum(x['value'] is not None for x in rows),rows=results)
 (out/('talk-'+mode+'.json')).write_text(json.dumps(report,indent=2)+'\n');reports.append(report)
 print(mode,report['passed'],'/',report['total'],flush=True)
assert all(x['passed']==x['total'] for x in reports)
