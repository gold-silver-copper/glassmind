# SPDX-License-Identifier: MIT
"""Recompute frozen-output grades and independently verify the four new answers."""
from pathlib import Path
from fractions import Fraction
from collections import Counter
import argparse,hashlib,json,sys,importlib.util
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);p.add_argument('--data',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
metric=Path(__file__).resolve().parents[2]/'typed-label-metric'
freeze=json.loads((metric/'FREEZE.json').read_text())
for name,h in freeze['files'].items():assert hashlib.sha256((metric/name).read_bytes()).hexdigest()==h
sys.path.insert(0,str(metric));from metric import parse_gold,compare
from replay import PARQUET,QHASH
legacy_path=metric.parent/'evaluate_mathqa.py';pins=json.loads((metric/'INPUTS.json').read_text());assert hashlib.sha256(legacy_path.read_bytes()).hexdigest()==pins[legacy_path.name]['sha256']
spec=importlib.util.spec_from_file_location('legacy',legacy_path);legacy=importlib.util.module_from_spec(spec);spec.loader.exec_module(legacy)
for split,(h,n) in PARQUET.items():
 b=(a.data/(split+'.parquet')).read_bytes();assert len(b)==n and hashlib.sha256(b).hexdigest()==h
import pyarrow.parquet as pq
rows=pq.read_table(a.data/'validation.parquet',columns=['Problem','options','correct']).to_pylist();assert len(rows)==4475
assert hashlib.sha256(''.join(json.dumps(r['Problem'])+'\n' for r in rows).encode()).hexdigest()==QHASH
old=[json.loads(x) for x in (a.run/'baseline/full.predictions.jsonl').read_text().splitlines()];new=[json.loads(x) for x in (a.run/'evidence/full/predictions.jsonl').read_text().splitlines()];assert len(old)==len(new)==len(rows)
counts={k:Counter() for k in ['baseline_v3','candidate_v3','baseline_legacy','candidate_legacy']};changed=[];old_answers=0
for i,(row,b,c) in enumerate(zip(rows,old,new)):
 h=hashlib.sha256(row['Problem'].encode()).hexdigest();assert b['index']==c['index']==i and b['question_sha256']==c['question_sha256']==h
 if b['actual']['status']=='answer':assert b['actual']==c['actual'];old_answers+=1
 if b['actual']!=c['actual'] and c['actual']['status']=='answer':changed.append(i)
 _,gold=parse_gold(row['options'],row['correct']);legacy_gold=legacy.numeric_option(row)
 for prefix,x in [('baseline',b['actual']),('candidate',c['actual'])]:
  counts[prefix+'_v3'][compare(gold,x)['grade']]+=1
  grade='refused' if x['status']=='refusal' else 'correct' if legacy_gold is not None and Fraction(x['value'])==legacy_gold else 'wrong';counts[prefix+'_legacy'][grade]+=1
assert old_answers==32 and changed==[390,706,1082,1828]
# Independent reading of these four saved question texts gives individual
# completion times in a common unit; all agents work together at constant rates.
times={390:([3,9,18],'hours'),706:([25,5,10],'days'),1082:([24,8,12],'days'),1828:([3,9,15],'days')}
audit=[]
for i,(ts,u) in times.items():
 rate=sum((Fraction(1,t) for t in ts),Fraction());expected=1/rate;actual=new[i]['actual'];assert Fraction(actual['value'])==expected and actual['unit']==u
 audit.append(dict(index=i,question=rows[i]['Problem'],individual_times=ts,net_rate=str(rate),completion_time=str(expected),unit=u))
assert counts['baseline_v3']==Counter(correct=29,refused=4443,wrong=3)
assert counts['candidate_v3']==Counter(correct=33,refused=4439,wrong=3)
assert counts['candidate_legacy']==Counter(correct=32,refused=4439,wrong=4)
result=dict(total=len(rows),counts={k:dict(v) for k,v in counts.items()},old_answer_payloads_preserved=old_answers,new_answers=audit,frozen_gate_pass=False,required_correct=34,source='Frozen MathQA validation; no inference or retuning in this audit.')
a.out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='new_answers'},indent=2))
