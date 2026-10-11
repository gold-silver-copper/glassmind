from pathlib import Path
import collections,hashlib,json,math,re,subprocess,sys
import pyarrow.parquet as pq
root=Path('/home/gsc/code/glassmind');work=Path('/home/gsc/.cache/glassmind/continuous-20261010');run=work/'equationproposals-run/frozen-candidate';out=Path(__file__).parent/'candidate-review'
sys.path.insert(0,str(root/'tools/quantities/typed-label-metric'));from metric import parse_gold,compare
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
verified={}
for name in ['prefit-manifest.json','calibration-prescore-manifest.json','sample-prescore-manifest.json','full-prescore-manifest.json','final-manifest.json']:
 for name,v in json.loads((run/name).read_text()).items():
  p=Path(name)
  if name not in verified:verified[name]=(p.stat().st_size,sha(p))
  assert verified[name]==(v['bytes'],v['sha256']),name
train=pq.read_table(work/'equation-source/train.parquet',columns=['Problem','options','correct']).to_pylist();dev=pq.read_table(work/'equation-source/validation.parquet',columns=['Problem','options','correct']).to_pylist()
groups=[hashlib.sha256(re.sub('[0-9]+','#',' '.join(r['Problem'].lower().split())).encode()).hexdigest() for r in train];fit=[i for i,g in enumerate(groups) if int(g,16)%5];cal=[i for i,g in enumerate(groups) if int(g,16)%5==0];split=json.loads((run/'split.json').read_text());assert split=={'fit':fit,'calibration':cal,'group_hashes':groups};assert not {groups[i] for i in fit}&{groups[i] for i in cal}
def load(n):return [json.loads(l) for l in (run/n).read_text().splitlines()]
def grade(row,pred):
 _,gold=parse_gold(row['options'],row['correct'])
 a={'status':'answer','value':pred['value'],'unit':None} if pred['status']=='proposed' else {'status':'refusal'}
 return compare(gold,a)['grade']
reports={};frontiers={}
for method in ['candidate','lexical','lexical_restricted']:
 d=load('full-'+method+'.jsonl');assert len(d)==4475
 counts=collections.Counter(grade(r,p) for r,p in zip(dev,d));reports[method]=dict(counts)
 c=load('calibration-'+method+'.jsonl');assert len(c)==5925
 byscore=collections.defaultdict(list)
 for i,p in zip(cal,c):
  if p['status']=='proposed':byscore[p['score']].append(grade(train[i],p)=='correct')
 n=correct=0;qualifying=[];best=None
 for score,ys in sorted(byscore.items(),reverse=True):
  n+=len(ys);correct+=sum(ys)
  if n>=30:
   precision=correct/n
   if precision>=.9:qualifying.append((score,n,correct))
   if best is None or (precision,n,correct,score)>tuple(best):best=[precision,n,correct,score]
 assert not qualifying
 frontiers[method]=dict(threshold=None,best_precision_min30=best)
 expected=json.loads((run/'full-report.json').read_text())[method]
 assert all(counts[k]==expected['raw'][k] for k in ['correct','refused','wrong','unscorable'])
 assert expected['selected']==0 and expected['threshold'] is None
# Class inventory can only come from cleaned FIT executions.
executions=load('train-executions.jsonl');assert len(executions)==len(train)
key=lambda p:json.dumps(p,sort_keys=True,separators=(',',':'))
clean=[i for i in fit if executions[i]['status']=='proposed' and grade(train[i],executions[i])=='correct']
counts=collections.Counter(key(executions[i]['program']) for i in clean)
bank=[p for p,n in sorted(counts.items(),key=lambda kv:(-kv[1],kv[0])) if n>=3][:512]
model=json.loads((run/'model.json').read_text());assert bank==[key(c['program']) for c in model['classes']]
assert set(model)=={'schema','provenance','idf','classes'} and all(set(c)=={'program','weights'} for c in model['classes'])
compat={}
for tag,n in [('sample',200),('full',4475),('synthetic',3116)]:
 a=run/('solve-'+tag+'-baseline.jsonl');b=run/('solve-'+tag+'-candidate.jsonl');assert a.read_bytes()==b.read_bytes() and len(a.read_text().splitlines())==n
 compat[tag]=dict(rows=n,sha256=sha(a))
order=sorted(range(len(dev)),key=lambda i:(hashlib.sha256(dev[i]['Problem'].encode()).hexdigest(),i))[:200];assert order==json.loads((run/'sample-indices.json').read_text())
qs=[dev[i]['Problem'] for i in order];data=''.join(json.dumps(q)+'\n' for q in qs)
p=subprocess.run([str(run/'glassmind-quantities'),'--propose-model',str(run/'model.json'),'--jsonl'],input=data,text=True,capture_output=True,check=True,timeout=60)
(out/'sample-replay.jsonl').write_text(p.stdout);assert p.stdout==(run/'sample-candidate.jsonl').read_text()
report=dict(manifest_paths_verified=len(verified),fit_rows=len(fit),calibration_rows=len(cal),fit_groups=len({groups[i] for i in fit}),calibration_groups=len({groups[i] for i in cal}),disjoint_groups=True,clean_fit=len(clean),unrestricted_programs=len(counts),candidate_programs=len(bank),raw_full=reports,calibration_frontiers=frontiers,default_solve_compatibility=compat,sample_replay_rows=200,sample_replay_identical=True)
(out/'metric-audit.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
