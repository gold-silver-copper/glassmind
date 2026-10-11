from pathlib import Path
from fractions import Fraction as F
from collections import Counter
import hashlib,importlib.util,json,subprocess,sys
root=Path('/home/gsc/code/glassmind');work=Path('/home/gsc/.cache/glassmind/continuous-20261010');run=work/'rateclauses-run';saved=run/'submitted/tools/quantities/rate-clauses/reports/full';metric=root/'tools/quantities/typed-label-metric'
for n,h in json.loads((metric/'FREEZE.json').read_text())['files'].items():assert hashlib.sha256((metric/n).read_bytes()).hexdigest()==h
sys.path.insert(0,str(metric));from metric import parse_gold,compare
from replay import PARQUET,QHASH
legacy_path=metric.parent/'evaluate_mathqa.py';pins=json.loads((metric/'INPUTS.json').read_text());assert hashlib.sha256(legacy_path.read_bytes()).hexdigest()==pins[legacy_path.name]['sha256'];spec=importlib.util.spec_from_file_location('legacy',legacy_path);legacy=importlib.util.module_from_spec(spec);spec.loader.exec_module(legacy)
data=work/'quantities-run/tmp/mathqa'
for split,(h,n) in PARQUET.items():
 b=(data/(split+'.parquet')).read_bytes();assert len(b)==n and hashlib.sha256(b).hexdigest()==h
import pyarrow.parquet as pq
rows=pq.read_table(data/'validation.parquet',columns=['Problem','options','correct']).to_pylist();assert len(rows)==4475;assert hashlib.sha256(''.join(json.dumps(r['Problem'])+'\n' for r in rows).encode()).hexdigest()==QHASH
old=[json.loads(l) for l in (run/'baseline/full.predictions.jsonl').read_text().splitlines()];newraw=(saved/'predictions.jsonl').read_text().splitlines(keepends=True);new=[json.loads(l) for l in newraw];ledger=[json.loads(l) for l in (saved/'ledger.jsonl').read_text().splitlines()];assert len(old)==len(new)==len(ledger)==4475
counts={k:Counter() for k in ['baseline_v3','candidate_v3','baseline_legacy','candidate_legacy']};changed=[];old_indices=[]
for i,(row,b,c,l) in enumerate(zip(rows,old,new,ledger)):
 assert b['index']==l['index']==i and b['question_sha256']==hashlib.sha256(row['Problem'].encode()).hexdigest();assert l['question']==row['Problem'] and l['baseline']==b['actual'] and l['candidate']==c
 if b['actual']['status']=='answer':assert b['actual']==c;old_indices.append(i)
 if b['actual']!=c and c['status']=='answer':changed.append(i)
 _,gold=parse_gold(row['options'],row['correct']);lg=legacy.numeric_option(row)
 for prefix,x in [('baseline',b['actual']),('candidate',c)]:
  counts[prefix+'_v3'][compare(gold,x)['grade']]+=1
  counts[prefix+'_legacy']['refused' if x['status']=='refusal' else 'correct' if lg is not None and F(x['value'])==lg else 'wrong']+=1
assert changed==[2469,3869,4126] and len(old_indices)==32
# Independent derivation from the three saved Problem texts: subtract known
# individual work/day rates from the stated combined work/day rate, then invert.
equations={2469:(4,[12,18]),3869:(6,[10]),4126:(1,[20])};audit=[]
for i,(joint,known) in equations.items():
 rate=F(1,joint)-sum((F(1,t) for t in known),F());answer=1/rate;assert F(new[i]['value'])==answer and new[i]['unit']=='days'
 _,gold=parse_gold(rows[i]['options'],rows[i]['correct']);audit.append(dict(index=i,question=rows[i]['Problem'],combined_completion_days=joint,known_individual_days=known,missing_rate_jobs_per_day=str(rate),missing_completion_days=str(answer),candidate=new[i],v3_grade=compare(gold,new[i])['grade'],selected_option=rows[i]['options'],correct_choice=rows[i]['correct']))
replay=subprocess.run([str(run/'baseline/quantities'),'--jsonl'],input=''.join(json.dumps(rows[i]['Problem'])+'\n' for i in old_indices),text=True,capture_output=True,check=True,timeout=60);assert replay.stdout.splitlines(keepends=True)==[newraw[i] for i in old_indices]
assert counts['baseline_v3']==Counter(correct=29,refused=4443,wrong=3);assert counts['candidate_v3']==Counter(correct=31,refused=4440,wrong=4);assert counts['candidate_legacy']==Counter(correct=30,refused=4440,wrong=5)
result=dict(total=4475,counts={k:dict(v) for k,v in counts.items()},old_answer_objects_and_raw_lines_preserved=32,new_answers=audit,gate_pass=False,required_correct=34,prediction_sha256=hashlib.sha256((saved/'predictions.jsonl').read_bytes()).hexdigest(),scope='Independent frozen-output rescore and rational derivation; no new candidate inference or tuning. Index4126 metric mismatch remains counted despite reciprocal-rate label discrepancy.')
(Path(__file__).parent/'full-audit.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='new_answers'},indent=2))
