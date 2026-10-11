from fractions import Fraction as F
from pathlib import Path
import json,subprocess,hashlib
root=Path('/home/gsc/.cache/glassmind/continuous-20261010')
binary=root/'coordinator/percentchange-review/glassmind-quantities'
out=root/'coordinator/percentchange-review/integrated'
rows=[]
def add(q,value=None,unit=None):rows.append(dict(question=q,value=None if value is None else str(value),unit=unit))
for base in [0,13,72]:
 for rate in [0,25,100,125]:
  for direction,sign in [('increased',1),('decreased',-1)]:
   factor=1+sign*F(rate,100)
   add(f'{base} is {direction} by {rate}% . find the final number',F(base)*factor if factor>=0 else None)
for rate,known in [(25,15),(50,18),(100,0),(125,9)]:
 for direction,sign in [('increased',1),('reduced',-1)]:
  factor=1+sign*F(rate,100)
  add(f'a number {direction} by {rate} percent gives {known} . the number is',F(known)/factor if factor>0 else None)
for first,second in [(25,20),(50,50),(10,10),(0,75),(100,40)]:
 for d1,s1 in [('increasing',1),('decreasing',-1)]:
  for d2,s2 in [('increasing',1),('decreasing',-1)]:
   change=((1+s1*F(first,100))*(1+s2*F(second,100))-1)*100
   query='increasing' if change>=0 else 'decreasing'
   add(f'{d1} the original cost of an engine by {first} percent and then {d2} the new cost by {second} percent is equivalent to {query} the original cost by what percentage',abs(change),'percent')
for initial,rate,years in [(64,25,2),(100,10,0),(80,50,2),(5,10,1)]:
 for direction,sign in [('increase',1),('decrease',-1)]:
  for when,inverse in [('before',True),('after',False)]:
   factor=(1+sign*F(rate,100))**years
   value=F(initial)/factor if inverse else F(initial)*factor
   add(f'the present population of city amber is {initial} . population {direction} rate is {rate} percent p . a . find the population of city {when} {years} years',value if value.denominator==1 else None)
for q in [
 'a number increased by 20 percentage points gives 120 . the number is',
 '70 is increased by 10% . find the final number and then add 4',
 'the present population of city amber is 80 . population increase rate is 25% p . a . find the population of town after 2 years',
 'increasing the original cost of an engine by 20 percent and then increasing the new price by 30 percent is equivalent to increasing the original cost by what percent',
 'increasing the original price of an engine by 20 percent and then increasing the original price by 30 percent is equivalent to increasing the original price by what percent',
 '-20 is increased by 10% . find the final number',
 '20 is increased by -10% . find the final number',
 '20 is increased by 10% . find the initial number',
 '20 is increased by 10% and reduced by 10% . find the final number',
 'the population of a city is 100 . it decreases annually at the rate of 125% . what will be its population after 0 years',
]:add(q)
(out/'frozen-cases.json').write_text(json.dumps(rows,indent=2)+'\n')
p=subprocess.run([str(binary),'--jsonl'],input=''.join(json.dumps(r['question'])+'\n' for r in rows),text=True,capture_output=True,check=True,timeout=60)
actual=[json.loads(s) for s in p.stdout.splitlines()];assert len(actual)==len(rows)
for row,a in zip(rows,actual):
 row['actual']=a
 row['pass']=a['status']=='refusal' if row['value'] is None else a['status']=='answer' and F(a['value'])==F(row['value']) and a['unit']==row['unit']
report=dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),total=len(rows),passed=sum(r['pass'] for r in rows),rows=rows)
(out/'independent.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='rows'}))
print(json.dumps([r for r in rows if not r['pass']],indent=2));assert all(r['pass'] for r in rows)
