from pathlib import Path
from fractions import Fraction
import argparse,copy,hashlib,json,re,subprocess
p=argparse.ArgumentParser();p.add_argument('--frozen',type=Path,required=True);a=p.parse_args();base=Path(__file__).parent;out=base/'candidate-review';out.mkdir(exist_ok=False)
for f,h in json.loads((base/'freeze.json').read_text()).items():
 if isinstance(h,str) and len(h)==64:assert hashlib.sha256((base/f).read_bytes()).hexdigest()==h
binary=a.frozen/'glassmind-quantities';executor=a.frozen/'equation_executor';model=a.frozen/'model.json';threshold=json.loads((a.frozen/'thresholds.json').read_text())['candidate']
def call(cmd,rows,label):
 proc=subprocess.run([str(x) for x in cmd],input=''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in rows),text=True,capture_output=True,timeout=30)
 (out/(label+'.stdout')).write_text(proc.stdout);(out/(label+'.stderr')).write_text(proc.stderr)
 return proc.returncode,[json.loads(l) for l in proc.stdout.splitlines() if l.strip()]
sem=json.loads((base/'semantic.json').read_text());rc,pred=call([binary,'--propose-model',model,'--jsonl'],[r['question'] for r in sem],'semantic');assert rc==0 and len(pred)==len(sem)
rows=[]
for c,r in zip(sem,pred):
 correct=r['status']=='proposed' and Fraction(r['value'])==Fraction(c['value'])
 selected=threshold is not None and r['status']=='proposed' and r['score']>=threshold
 rows.append(dict(**c,result=r,correct=correct,selected=selected))
(out/'semantic.json').write_text(json.dumps(rows,indent=2,ensure_ascii=False)+'\n')
constants=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
def program(text):
 steps=[]
 for token in text.split('|'):
  op,args=token.split('(',1);left,right=args[:-1].split(',')
  def arg(t):
   if t.startswith('n'):return dict(kind='input',index=int(t[1:]))
   if t.startswith('#'):return dict(kind='step',index=int(t[1:]))
   return dict(kind='constant',index=constants.index(t[6:].replace('_','.')) if t[6:].replace('_','.') in constants else 255)
  steps.append(dict(op=op,left=arg(left),right=arg(right)))
 return dict(steps=steps)
contracts={}
for filename in ['bindings.json','invalid-programs.json','input-contracts.json']:
 checks=[]
 for c in json.loads((base/filename).read_text()):
  rc,rs=call([executor],[dict(question=c['question'],program=program(c['program']))],filename[:-5]+'-'+c['name']);r=rs[0] if rs else None
  rejected=rc!=0 or (r and r.get('status')=='refusal')
  if filename=='bindings.json':
   if c['value'] is None:ok=rejected
   else:
    spans=[dict(text=n['literal'],start=n['start'],end=n['end']) for n in r.get('numbers',[])] if r else []
    ok=rc==0 and r['status']=='proposed' and Fraction(r['value'])==Fraction(c['value']) and spans==c['expected_spans']
  else:ok=bool(rejected)
  checks.append(dict(name=c['name'],pass_=bool(ok),exit=rc,result=r))
 contracts[filename]=checks
m=json.loads(model.read_text());bounds=[]
variants={}
def change(name,fn):
 v=copy.deepcopy(m);fn(v);variants[name]=json.dumps(v).encode()
change('unknown_schema',lambda v:v.update(schema='unknown'))
change('wrong_idf_length',lambda v:v.update(idf=[]))
change('nan_weight',lambda v:v['classes'][0]['weights'].update({'0':float('nan')}))
change('out_of_range_weight',lambda v:v['classes'][0]['weights'].update({'8192':0.1}))
change('unknown_operation',lambda v:v['classes'][0]['program']['steps'][0].update(op='system'))
change('forward_reference',lambda v:v['classes'][0]['program']['steps'][0].update(left=dict(kind='step',index=0)))
change('oversized_provenance',lambda v:v.update(provenance='x'*1025))
compact=json.dumps(m,separators=(',',':')).encode()
variants['over_32MiB']=compact+b' '*(32*1024*1024+1-len(compact))
variants['over_decimal_32MB']=compact+b' '*(32000001-len(compact))
for name,data in variants.items():
 path=out/(name+'.json');path.write_bytes(data)
 rc,rs=call([binary,'--propose-model',path,'What is 2 plus 3?'],[],name)
 bounds.append(dict(name=name,bytes=len(data),exit=rc,rejected=rc!=0,result=rs))
summary=dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),semantic_correct=sum(r['correct'] for r in rows),semantic_refused=sum(r['result']['status']=='refusal' for r in rows),semantic_total=len(rows),selected=sum(r['selected'] for r in rows),semantics_explicit=all(r['result'].get('semantics')=='unverified' for r in rows if r['result']['status']=='proposed'),contracts={k:dict(passed=sum(r['pass_'] for r in v),total=len(v)) for k,v in contracts.items()},model_bounds=bounds)
(out/'contracts.json').write_text(json.dumps(contracts,indent=2)+'\n');(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
