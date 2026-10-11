"""Authored public development probes; fixed before DEV inference."""
import json,subprocess,sys
from fractions import Fraction as F
from pathlib import Path
cases=[]
def add(q,v=None,u=None):cases.append(dict(question=q,expected=None if v is None else str(v),unit=u))
for item,p,q in [('lantern',10,10),('violin',25,20),('basket',12,15),('kettle',0,30)]:
 add(f'Increasing the original price of a {item} by {p} percent and then increasing the new price by {q} percent is equivalent to increasing the original price by what percent?',(F(100+p,100)*F(100+q,100)-1)*100,'percent')
for initial,p in [(75,20),(120,25),(0,10),(200,100)]:
 add(f'{initial} is increased by {p}%. Find the final number.',F(initial*(100+p),100))
 add(f'A number increased by {p}% gives {F(initial*(100+p),100)}. The number is',initial)
for p,q in [(20,20),(50,50),(30,10)]:
 add(f'A number is increased by {p}% and then decreased by {q}%. What is the percentage '+('decrease?' if p==q else 'increase?'),abs((F(100+p,100)*F(100-q,100)-1)*100),'percent')
for name,initial,rate,years in [('mira',10000,10,2),('luma',8000,20,3),('cedar',1000,50,2)]:
 final=F(initial)*F(100+rate,100)**years
 add(f'The population of town {name} is {initial}. It increases annually at the rate of {rate}%. What will be its population after {years} years?',final)
 add(f'The present population of town {name} is {final}. Population increase rate is {rate}% p. a. Find the population of town before {years} years?',initial)
add('90 is decreased by 100%. Find the final number.',0)
for text in [
'A number decreased by 100% gives 0. The number is',
'-20 is increased by 10%. Find the final number.',
'20 is increased by -10%. Find the final number.',
'20 is decreased by 101%. Find the final number.',
'20 is increased by 10 percentage points. Find the final number.',
'20 is increased by 10%. Find the final number. Also add 5.',
'A number increased by 20% gives 120. The number is 90. The number is',
'Increasing the original price of a lantern by 10 percent and then increasing the new cost by 10 percent is equivalent to increasing the original price by what percent?',
'Increasing the original price of a lantern by 10 percent and then increasing the original price by 10 percent is equivalent to increasing the original price by what percent?',
'The present population of a city is 1440. Population increase rate is 20% p. a. Find the population of village before 2 years?',
'The population of a town is 1. It increases annually at the rate of 10%. What will be its population after 1 year?',
'A number is increased by x percent and then decreased by y percent. What is the percentage increase?',
'20 dollars is increased by 10%. Find the final number.',
'20 is changed by 10%. Find the final number.',
'A number is increased by 20% and then decreased by 20%. What is the percentage increase?',
]:add(text)
add('What is 20 percent of 120?',24)
for name,binary in [('baseline',sys.argv[1]),('candidate',sys.argv[2])]:
 r=subprocess.run([binary,'--jsonl'],input=''.join(json.dumps(c['question'])+'\n' for c in cases),capture_output=True,text=True,check=True,timeout=120)
 answers=[json.loads(x) for x in r.stdout.splitlines()];assert len(answers)==len(cases)
 for c,a in zip(cases,answers):
  ok=(a['status']=='refusal') if c['expected'] is None else a['status']=='answer' and F(a['value'])==F(c['expected']) and a.get('unit')==c['unit']
  c[name]=dict(actual=a,passed=ok)
report=dict(total=len(cases),baseline_pass=sum(c['baseline']['passed'] for c in cases),candidate_pass=sum(c['candidate']['passed'] for c in cases),cases=cases,source='authored development probes v1',license='MIT OR Apache-2.0')
Path(sys.argv[3]).write_text(json.dumps(report,indent=2)+'\n')
print({k:v for k,v in report.items() if k!='cases'})
assert report['candidate_pass']==len(cases)
