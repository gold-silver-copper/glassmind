import json,subprocess,sys
from pathlib import Path
from fractions import Fraction
cases=json.loads(Path('tools/quantities/percent-change/parent/frozen-cases.json').read_text())
r=subprocess.run([sys.argv[1],'--jsonl'],input=''.join(json.dumps(c['question'])+'\n' for c in cases),text=True,capture_output=True,check=True,timeout=120)
answers=[json.loads(x) for x in r.stdout.splitlines()];assert len(answers)==len(cases)==78
for c,a in zip(cases,answers):c.update(actual=a,passed=(a['status']=='refusal' if c['value'] is None else a['status']=='answer' and Fraction(a['value'])==Fraction(c['value']) and a.get('unit')==c['unit']))
report=dict(total=len(cases),passed=sum(c['passed'] for c in cases),cases=cases)
Path(sys.argv[2]).write_text(json.dumps(report,indent=2)+'\n');print({k:v for k,v in report.items() if k!='cases'});assert report['passed']==report['total']
