#!/usr/bin/env python3
"""Post-freeze English audit: independently transcribed extents and rates.

Not a runtime answer table. Never imported by Rust or either evaluator.
The six new answers were individually reviewed after the frozen full run.
"""
import argparse, hashlib, json
from fractions import Fraction as F
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--baseline',type=Path,required=True);p.add_argument('--candidate',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
b=json.loads(a.baseline.read_text());c=json.loads(a.candidate.read_text())
before={r['index']:r for r in b['answered']};after={r['index']:r for r in c['answered']}
# From each English Problem, never from gold or proposed output:
# row, train metres, bridge metres, speed magnitude, explicit speed unit.
premises=[(295,100,140,36,'km/h'),(563,150,150,36,'km/h'),(1128,60,120,30,'m/s'),(1986,100,170,36,'km/h'),(2836,140,150,36,'km/h'),(2873,120,150,36,'km/h')]
assert set(after)-set(before)=={r[0] for r in premises}
audit=[]
for index,length,bridge,speed,unit in premises:
    # Full crossing: front moves train length + bridge length while rear clears.
    # English explicitly requests seconds and supplies the speed for the event.
    meters_per_second=F(speed)* (F(1000,3600) if unit=='km/h' else F(1))
    seconds=F(length+bridge)/meters_per_second
    row=after[index]
    assert row['actual']['unit']=='seconds'
    assert F(row['actual']['value'])==seconds
    audit.append({'index':index,'question':row['question'],'transcribed_train_m':length,'transcribed_bridge_m':bridge,'transcribed_speed':speed,'transcribed_speed_unit':unit,'independent_m_per_s':str(meters_per_second),'independent_seconds':str(seconds),'actual':row['actual'],'scored_outcome':row['outcome'],'semantic_review':'Unique complete-crossing duration from both explicit extents and the explicit event speed. No unit default, option, rationale or formula input.','gold_normalizer_failed':row['expected']=='None'})
losses=[{'index':i,'before':r,'after':after.get(i)} for i,r in before.items() if r!=after.get(i)]
r={'decision':'REJECT','reason':'Only +1 scored correct, below +3; five new scored wrong. Unparseable gold remains wrong by frozen metric. No retuning.','baseline':{k:b[k] for k in ['total','right','refused','wrong','non_numeric_gold','binary_sha256']},'candidate':{k:c[k] for k in ['total','right','refused','wrong','non_numeric_gold','binary_sha256']},'baseline_records_preserved':len(before)-len(losses),'losses':losses,'new_answer_audit':audit,'transitions':{'refused_to_right':1,'refused_to_wrong':5,'unchanged_right':22,'unchanged_wrong':3,'unchanged_refused':4444},'new_semantic_errors_detected':0,'scope':'All six new dev answers independently recomputed from English using Fraction; local probes are authored development diagnostics, not independent held-out evaluation.'}
a.output.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k!='new_answer_audit'},indent=2));assert not losses
