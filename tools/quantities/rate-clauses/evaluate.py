"""Frozen adapter: Problem-only inference saved before loading DEV labels.
Imports both unchanged evaluators; no label interpretation rules here.
"""
import argparse,collections,datetime,hashlib,importlib.util,json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE.parent/'typed-label-metric'))
from metric import parse_gold,compare
spec=importlib.util.spec_from_file_location('legacy',HERE.parent/'evaluate_mathqa.py')
legacy=importlib.util.module_from_spec(spec);spec.loader.exec_module(legacy)
from fractions import Fraction

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,x):p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--data',type=Path,required=True);ap.add_argument('--baseline',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--sample',action='store_true');a=ap.parse_args()
 a.output.mkdir(parents=True,exist_ok=True)
 freeze=json.loads((HERE/'FREEZE.json').read_text())
 for name,h in freeze['files'].items(): assert sha(Path(name))==h, name
 assert sha(a.binary)==freeze['candidate_binary_sha256']
 import pyarrow.parquet as pq
 questions=pq.read_table(a.data/'validation.parquet',columns=['Problem']).column(0).to_pylist();assert len(questions)==4475
 n=100 if a.sample else 4475;qs=questions[:n]
 pred=a.output/'predictions.jsonl';assert not pred.exists(),'do not overwrite inference'
 run=subprocess.run([str(a.binary),'--jsonl'],input=''.join(json.dumps(q)+'\n' for q in qs),text=True,capture_output=True,check=True,timeout=180)
 raw=run.stdout.splitlines(keepends=True);actual=[json.loads(x) for x in raw];assert len(actual)==n
 pred.write_text(run.stdout)
 write(a.output/'inference.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),problem_only=True,count=n,sha256=sha(pred)))
 # Label access starts only after saved Problem-only predictions.
 labels=pq.read_table(a.data/'validation.parquet',columns=['options','correct']).to_pylist()
 baseline=[json.loads(l)['actual'] for l in (a.baseline/'full.predictions.jsonl').read_text().splitlines()];assert len(baseline)==4475
 old_indices=[i for i,b in enumerate(baseline[:n]) if b['status']=='answer']
 oldrun=subprocess.run([str(a.baseline/'quantities'),'--jsonl'],input=''.join(json.dumps(qs[i])+'\n' for i in old_indices),text=True,capture_output=True,check=True,timeout=60)
 oldraw=oldrun.stdout.splitlines(keepends=True)
 assert len(oldraw)==len(old_indices)
 raw_preserved=all(old==raw[i] for i,old in zip(old_indices,oldraw))
 write(a.output/'old-payload-byte-check.json',dict(indices=old_indices,identical=raw_preserved,baseline_sha256=sha(a.baseline/'quantities')))
 scores={k:collections.Counter() for k in ['baseline','candidate']};old={k:collections.Counter() for k in scores};changes=[];ledger=[];unchanged=True;newbad=[]
 for i,(q,b,c) in enumerate(zip(qs,baseline,actual)):
  _,g=parse_gold(labels[i]['options'],labels[i]['correct']); lg=legacy.numeric_option(labels[i]);entry=dict(index=i,question=q,baseline=b,candidate=c)
  for name,x in [('baseline',b),('candidate',c)]:
   grade=compare(g,x)['grade'];scores[name][grade]+=1;entry[name+'_grade']=grade
   outcome='refused' if x['status']=='refusal' else ('correct' if lg is not None and Fraction(x['value'])==lg else 'wrong');old[name][outcome]+=1
  if b['status']=='answer':unchanged &= b==c
  if b['status']=='refusal' and c['status']=='answer':
   changes.append(entry)
   if entry['candidate_grade']!='correct':newbad.append(i)
  ledger.append(entry)
 summary=dict(total=n,scores=scores,legacy_scores=old,old_answer_payloads_identical=unchanged,old_raw_answer_lines_identical=raw_preserved,old_answer_count=sum(b['status']=='answer' for b in baseline[:n]),new_answered=len(changes),new_bad=newbad,gate=n==4475 and scores['candidate']['correct']>=34 and not newbad and unchanged and raw_preserved)
 write(a.output/'summary.json',summary);write(a.output/'new-answers.json',changes)
 (a.output/'ledger.jsonl').write_text(''.join(json.dumps(x,sort_keys=True)+'\n' for x in ledger))
 print(json.dumps(summary,indent=2))
if __name__=='__main__':main()
