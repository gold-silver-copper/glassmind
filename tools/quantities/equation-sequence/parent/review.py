from pathlib import Path
from fractions import Fraction
import collections,copy,hashlib,json,re,subprocess,sys
import pyarrow.parquet as pq
root=Path('/home/gsc/code/glassmind');work=Path('/home/gsc/.cache/glassmind/continuous-20261010');run=work/'equationsequence-run/tmp/equation-sequence';old=work/'equationproposals-run/frozen-candidate';fixtures=work/'coordinator/equation-proposal-independent';out=Path(__file__).parent/'results';out.mkdir(exist_ok=False)
sys.path.insert(0,str(root/'tools/quantities/typed-label-metric'));from metric import parse_gold,compare
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();load=lambda p:[json.loads(l) for l in p.read_text().splitlines()]
verified={}
for manifest in ['prefit-manifest.json','postfit-manifest.json','calibration-prescore-manifest.json','sample-prescore-manifest.json','full-prescore-manifest.json','final-manifest.json']:
 for name,v in json.loads((run/manifest).read_text()).items():
  p=Path(name)
  if name not in verified:verified[name]=(p.stat().st_size,sha(p))
  assert verified[name]==(v['bytes'],v['sha256']),name
for n,v in json.loads((fixtures/'freeze.json').read_text()).items():assert sha(fixtures/n)==v['sha256'] and (fixtures/n).stat().st_size==v['bytes']
assert sha(fixtures/'input-contracts.json')==json.loads((fixtures/'input-contracts-freeze.json').read_text())['sha256']
train=pq.read_table(work/'equation-source/train.parquet',columns=['Problem','options','correct']).to_pylist();dev=pq.read_table(work/'equation-source/validation.parquet',columns=['Problem','options','correct']).to_pylist()
split=json.loads((run/'split.json').read_text());assert split==json.loads((old/'split.json').read_text());groups=[hashlib.sha256(re.sub('[0-9]+','#',' '.join(r['Problem'].lower().split())).encode()).hexdigest() for r in train];assert groups==split['group_hashes'];assert split['calibration']==[i for i,g in enumerate(groups) if int(g,16)%5==0]
assert load(run/'train-executions.jsonl')==load(old/'train-executions.jsonl')
def grade(row,pred):
 _,gold=parse_gold(row['options'],row['correct'])
 a={'status':'answer','value':pred['value'],'unit':None} if pred['status']=='proposed' else {'status':'refusal'}
 return compare(gold,a)['grade']
full=load(run/'full-candidate.jsonl');assert len(full)==4475;grades=[grade(r,p) for r,p in zip(dev,full)];counts=collections.Counter(grades);expected=json.loads((run/'full-report.json').read_text())['candidate']['raw'];assert all(counts[k]==v for k,v in expected.items())
cal=load(run/'calibration-candidate.jsonl');assert len(cal)==5925;by=collections.defaultdict(list)
for i,p in zip(split['calibration'],cal):
 if p['status']=='proposed':by[p['score']].append(grade(train[i],p)=='correct')
n=correct=0;best=None;valid=[]
for score,ys in sorted(by.items(),reverse=True):
 n+=len(ys);correct+=sum(ys)
 if n>=30:
  v=(correct/n,n,correct,score)
  if best is None or v>best:best=v
  if correct/n>=.9:valid.append(v)
assert not valid and json.loads((run/'thresholds.json').read_text())['candidate'] is None
key=lambda p:json.dumps(p,sort_keys=True,separators=(',',':'))
execs=load(run/'train-executions.jsonl');clean=[i for i in split['fit'] if execs[i]['status']=='proposed' and grade(train[i],execs[i])=='correct'];bank={key(execs[i]['program']) for i in clean};novel=[i for i,p in enumerate(full) if p['status']=='proposed' and key(p['program']) not in bank]
composition=dict(clean_fit=len(clean),fit_structures=len(bank),novel_rows=len(novel),distinct_novel_programs=len({key(full[i]['program']) for i in novel}),novel_correct=sum(grades[i]=='correct' for i in novel),max_steps=max(len(p['program']['steps']) for p in full if p['status']=='proposed'))
binary=run/'equation_sequence';model=run/'model.json';executor=run/'equation_executor'
def call(cmd,rows,label):
 p=subprocess.run([str(x) for x in cmd],input=''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in rows),text=True,capture_output=True,timeout=45)
 (out/(label+'.stdout')).write_text(p.stdout);(out/(label+'.stderr')).write_text(p.stderr)
 return p.returncode,[json.loads(l) for l in p.stdout.splitlines() if l.strip()]
semantic=json.loads((fixtures/'semantic.json').read_text());rc,ss=call([binary,model],[c['question'] for c in semantic],'semantic');assert rc==0 and len(ss)==24
sr=[dict(**c,result=p,correct=p['status']=='proposed' and Fraction(p['value'])==Fraction(c['value']),selected=False) for c,p in zip(semantic,ss)];(out/'semantic.json').write_text(json.dumps(sr,indent=2,ensure_ascii=False)+'\n')
constants=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
def program(s):
 steps=[]
 for token in s.split('|'):
  op,rest=token.split('(',1);args=rest[:-1].split(',')
  def arg(t):
   if t.startswith('n'):return dict(kind='input',index=int(t[1:]))
   if t.startswith('#'):return dict(kind='step',index=int(t[1:]))
   v=t[6:].replace('_','.');return dict(kind='constant',index=constants.index(v) if v in constants else 255)
  steps.append(dict(op=op,left=arg(args[0]),right=arg(args[1])))
 return dict(steps=steps)
contracts={}
for file in ['bindings.json','invalid-programs.json','input-contracts.json']:
 rs=[]
 for c in json.loads((fixtures/file).read_text()):
  rc,preds=call([executor],[dict(question=c['question'],program=program(c['program']))],file[:-5]+'-'+c['name']);r=preds[0] if preds else None;refused=rc!=0 or (r and r['status']=='refusal')
  if file=='bindings.json' and c['value'] is not None:
   spans=[dict(text=n['literal'],start=n['start'],end=n['end']) for n in r.get('numbers',[])] if r else []
   ok=bool(r and r['status']=='proposed' and Fraction(r['value'])==Fraction(c['value']) and spans==c['expected_spans'])
  else:ok=bool(refused)
  rs.append(dict(name=c['name'],pass_=ok,exit=rc,result=r))
 contracts[file]=rs
(out/'contracts.json').write_text(json.dumps(contracts,indent=2)+'\n')
# One frozen sample replay, not a repeated full fit/evaluation.
indices=json.loads((run/'sample-indices.json').read_text());assert indices==json.loads((old/'sample-indices.json').read_text());rc,replay=call([binary,model],[dev[i]['Problem'] for i in indices],'sample-replay');assert rc==0 and replay==load(run/'sample-candidate.jsonl')
compat={}
for tag,n in [('sample',200),('full',4475),('synthetic',3116)]:
 p=run/('solve-'+tag+'-candidate.jsonl');b=old/('solve-'+tag+'-baseline.jsonl');assert p.read_bytes()==b.read_bytes() and len(load(p))==n;compat[tag]=dict(rows=n,sha256=sha(p))
m=json.loads(model.read_text());assert set(m)=={'schema','provenance','weights'};assert [len(w) for w in m['weights']]==[5*8192,42*8192,42*8192]
bounds=[]
for name in ['unknown_schema','head_count','weight_count','nan','oversized_provenance','decimal_size_limit']:
 v=copy.deepcopy(m)
 if name=='unknown_schema':v['schema']='invalid'
 elif name=='head_count':v['weights'].pop()
 elif name=='weight_count':v['weights'][0].pop()
 elif name=='nan':v['weights'][0][0]=float('nan')
 elif name=='oversized_provenance':v['provenance']='x'*1025
 data=json.dumps(v,separators=(',',':')).encode()
 if name=='decimal_size_limit':data+=b' '*(32000001-len(data))
 path=out/(name+'.json');path.write_bytes(data);rc,preds=call([binary,path],['What is 2 plus 3?'],name);bounds.append(dict(name=name,rejected=rc!=0,bytes=len(data)))
report=dict(disposition='rejected',manifest_paths_verified=len(verified),raw_full=dict(counts),calibration_best_min30=best,calibration_threshold=None,composition=composition,semantic_correct=sum(r['correct'] for r in sr),semantic_refused=sum(r['result']['status']=='refusal' for r in sr),semantic_wrong=sum(not r['correct'] and r['result']['status']=='proposed' for r in sr),semantic_total=24,selected=0,semantics_and_model_hash=all(r['result']['semantics']=='unverified' and r['result']['model_sha256']==sha(model) for r in sr if r['result']['status']=='proposed'),contracts={k:dict(passed=sum(r['pass_'] for r in v),total=len(v)) for k,v in contracts.items()},model_bounds=bounds,default_solve=compat,sample_replay_identical=True,binary_sha256=sha(binary),model_sha256=sha(model));(out/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
