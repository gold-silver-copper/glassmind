"""Offline evaluation: frozen metric, Problem-only inference, save before scoring."""
import argparse,hashlib,importlib.util,json,subprocess,sys
from pathlib import Path
from collections import Counter
from fractions import Fraction
HERE=Path(__file__).resolve().parent;METRIC=HERE.parent/'typed-label-metric'
sys.path.insert(0,str(METRIC));from metric import parse_gold,compare
from replay import PARQUET,QHASH

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,x):p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--data',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--sample',action='store_true');ap.add_argument('--score-only',action='store_true');a=ap.parse_args()
 a.output.mkdir(parents=True,exist_ok=True)
 freeze=json.loads((METRIC/'FREEZE.json').read_text())
 for name,h in freeze['files'].items():assert sha(METRIC/name)==h,(name,'freeze')
 for split,(h,n) in PARQUET.items():
  p=a.data/(split+'.parquet');assert sha(p)==h and p.stat().st_size==n
 import pyarrow.parquet as pq
 # Inference opens Problem only; labels are loaded only after predictions saved.
 questions=pq.read_table(a.data/'validation.parquet',columns=['Problem']).column(0).to_pylist();assert len(questions)==4475
 assert hashlib.sha256(''.join(json.dumps(q)+'\n' for q in questions).encode()).hexdigest()==QHASH
 limit=100 if a.sample else 4475;qs=questions[:limit];pred=a.output/'predictions.jsonl'
 if not a.score_only:
  assert not pred.exists(),'never overwrite completed inference'
  r=subprocess.run([str(a.binary),'--jsonl'],input=''.join(json.dumps(q)+'\n' for q in qs),text=True,capture_output=True,check=True,timeout=180)
  answers=[json.loads(x) for x in r.stdout.splitlines()];assert len(answers)==limit
  pred.write_text(''.join(json.dumps(dict(index=i,question_sha256=hashlib.sha256(q.encode()).hexdigest(),actual=x),sort_keys=True)+'\n' for i,(q,x) in enumerate(zip(qs,answers))))
  write(a.output/'inference.json',dict(binary_sha256=sha(a.binary),predictions_sha256=sha(pred),count=limit,full_questions_sha256=QHASH))
 saved=[json.loads(x) for x in pred.read_text().splitlines()];assert len(saved)==limit
 for i,x in enumerate(saved):assert x['index']==i and x['question_sha256']==hashlib.sha256(qs[i].encode()).hexdigest()
 inputs=json.loads((METRIC/'INPUTS.json').read_text());legacy_path=HERE.parent/'evaluate_mathqa.py';assert sha(legacy_path)==inputs[legacy_path.name]['sha256']
 spec=importlib.util.spec_from_file_location('legacy',legacy_path);legacy=importlib.util.module_from_spec(spec);spec.loader.exec_module(legacy)
 goldrows=pq.read_table(a.data/'validation.parquet',columns=['options','correct']).to_pylist()
 old=[legacy.numeric_option(r) for r in goldrows];assert sum(x is None for x in old)==1229
 series={}
 for name,file,n in [('production','full.predictions.jsonl',32)]:
  p=Path(__file__).resolve().parents[4]/'workrates-run'/'baseline'/file
  assert sha(p)=='98447f585880c5600b76b6bd949c4d7bfbb1f0754825ce2ee3404ef745813ebc'
  records=[json.loads(line) for line in p.read_text().splitlines()];assert len(records)==4475
  for i,v in enumerate(records):assert v['index']==i and v['question_sha256']==hashlib.sha256(questions[i].encode()).hexdigest()
  series[name]={v['index']:v['actual'] for v in records if v['actual']['status']=='answer'}
  assert len(series[name])==n
 ledger=[];scores={n:Counter() for n in ['production','candidate']};oldscores={n:Counter() for n in scores};changes=[]
 for i in range(limit):
  _,g=parse_gold(goldrows[i]['options'],goldrows[i]['correct']);entry=dict(index=i,question=qs[i],series={})
  for n in scores:
   actual=saved[i]['actual'] if n=='candidate' else series[n].get(i,dict(status='refusal'))
   grade=compare(g,actual)['grade'];scores[n][grade]+=1
   og='refused' if actual['status']=='refusal' else ('correct' if old[i] is not None and Fraction(actual['value'])==old[i] else 'wrong');oldscores[n][og]+=1
   entry['series'][n]=dict(actual=actual,grade=grade,legacy_grade=og)
  ledger.append(entry)
  if entry['series']['candidate']['actual']['status'] != entry['series']['production']['actual']['status'] or (entry['series']['candidate']['actual']['status']=='answer' and entry['series']['candidate']['actual']!=entry['series']['production']['actual']):changes.append(entry)
 lost={n:[e['index'] for e in ledger if e['series'][n]['grade']=='correct' and e['series']['candidate']['grade']!='correct'] for n in series}
 newbad=[e['index'] for e in ledger if e['series']['candidate']['grade'] in ['wrong','unscorable'] and e['index'] not in series['production']]
 preserved=all(saved[i]['actual']==a for i,a in series['production'].items() if i<limit)
 summary=dict(original_answers_unchanged=preserved,total=limit,full_total=4475,legacy_unparseable_full=1229,scores=scores,legacy_scores=oldscores,correct_losses=lost,new_bad=newbad,transitions={n:dict(Counter(e['series'][n]['grade']+' -> '+e['series']['candidate']['grade'] for e in ledger)) for n in series},gate=preserved and limit==4475 and scores['candidate']['correct']>=34 and not any(lost.values()) and not newbad,baseline_limit='Supplied current full4475 predictions; full hash/order and all32 answer records verified',metric='exact-label-comparison-v3')
 write(a.output/'summary.json',summary);write(a.output/'changes.json',changes)
 (a.output/'ledger.jsonl').write_text(''.join(json.dumps(e,sort_keys=True)+'\n' for e in ledger))
 print(json.dumps(summary,indent=2))
if __name__=='__main__':main()
