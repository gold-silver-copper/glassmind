#!/usr/bin/env python3
"""Authored independent English/physical probes; Fraction oracle, no gold lookup."""
import argparse, hashlib, json, subprocess
from fractions import Fraction as F
from itertools import permutations
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
cases=[]
def add(q,v=None,u=None):cases.append(dict(question=q,expected=None if v is None else str(v),unit=u))
for name,length,extent,speed in [('iris',F(35),F(90),F(18)),('sable',F(25,2),F(137,2),F(81,2)),('taro',F(501),F(299),F(144))]:
    # km/h is exactly 1000 m / 3600 s. Sum extents then divide speed.
    expected=(length+extent)/(speed*F(1000,3600))
    facts=[f'The length of the tram named {name} is {length} meters',f'The speed of the tram named {name} is {speed} km per hour']
    for order in permutations(facts):
        add(f'A tram named {name} is {length} meters long. '+'. '.join(order)+f'. In how many seconds will the tram named {name} cross a bridge {extent} meters long?',expected,'seconds')
    add(f'In how many seconds will a {length} meter long tram named {name} moving at {speed} km/h cross a {extent} meter long bridge?',expected,'seconds')
    add(f'A tram named {name} is {length} meters long. The speed of the tram named {name} is {speed} km/h. In how many seconds will the tram named {name} cross a bridge {extent} meters long?'.replace('named '+name+' cross','named wrong cross'))
    add(f'A tram named {name} is {length} meters long. The speed of the tram is {speed} km/h. The length of the tram is {length+1} meters. In how many seconds will it cross a pole?')
# Two unknowns: extent inferred from a linked event, then a new duration.
for speed,prior,bridge in [(F(27),F(16),F(300)),(F(72),F(11,2),F(220))]:
    add(f'A train running at {speed} km/h crosses a pole in {prior} seconds. In how many minutes will it cross a bridge {bridge} meters long at the same speed?',(prior+bridge/(speed*F(5,18)))/60,'minutes')
    add(f'A train running at {speed} km/h crosses a pole in {prior} seconds. In how many minutes will it cross a bridge {bridge} meters long?')
    add(f'A train 1 meter long running at {speed} km/h crosses a pole in {prior} seconds. In how many minutes will it cross a bridge {bridge} meters long at the same speed?')
add('A bus 30 feet long is moving at 45 mph. In how many seconds will the bus cross a bridge 90 yards long?',F(300,66),'seconds')
add('A car 4 meters long is moving at 10 meters per second. In how many seconds will it pass a pole?',F(2,5),'seconds')
base='A train 200 meters long is running at 36 km/h. In how many seconds will it pass a bridge 300 meters long?'
add(base,50,'seconds')
for old,new in [('36 km/h','0 km/h'),('36 km/h','-36 km/h'),('200 meters','-200 meters'),('200 meters','200 celsius'),('36 km/h','36 kg/h'),('seconds','kilograms'),('bridge 300 meters long','bridge'),('will it','will the bus'),('running','accelerating'),('long?','long while accelerating?')]:add(base.replace(old,new))
add(base+' The train stops halfway.')
add(base+' The speed of the train is 72 km/h.')
add('A train 200 meters long is running at 36 km/h. A train 100 meters long is running at 36 km/h. In how many seconds will it cross a pole?')
add('A train 200 meters long is running at 36 km/h. A platform is 50 meters long. A platform is 100 meters long. In how many seconds will it cross the platform?')
add('A train 200 meters long is running at 36 km/h. In how many seconds will it pass an electric platform 300 meters long?')
add('A train 200 meters long is running at 36 km/h. In how many seconds will it pass a bridge 300 meters long moving at 5 km/h?')
add(base.replace('200','1/0'))
add(base.replace('200','999999999999999999999999999999999999999999999'))
# Branch regression: unordered absolute difference retains two roots.
add('The difference between a number and 2 is 5. What is the number?')
stream=''.join(json.dumps(c['question'])+'\n' for c in cases)
r=subprocess.run([str(a.binary.resolve()),'--jsonl'],input=stream,text=True,capture_output=True,check=True,timeout=120)
answers=[json.loads(x) for x in r.stdout.splitlines()];assert len(answers)==len(cases)
fail=[]
for c,v in zip(cases,answers):
    okay=v['status']=='refusal' if c['expected'] is None else v['status']=='answer' and F(v['value'])==F(c['expected']) and v['unit']==c['unit']
    if not okay:fail.append(dict(**c,actual=v))
report={'source':'ratewords authored physical probes v1','license':'MIT OR Apache-2.0','total':len(cases),'answer_cases':sum(c['expected'] is not None for c in cases),'failures':fail,'cases':cases,'binary_sha256':hashlib.sha256(a.binary.read_bytes()).hexdigest()}
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'},indent=2));assert not fail
