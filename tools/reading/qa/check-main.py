#!/usr/bin/env python3
"""Check passage command input/output contracts, independently of answer accuracy."""
import argparse, hashlib, json, pathlib, subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=pathlib.Path,required=True);p.add_argument('--model',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=False);a.binary=a.binary.resolve();a.model=a.model.resolve()
context="Élodie visited Zürich in 2018."
passage=a.output/'passage.txt';passage.write_text(context)
base=[str(a.binary),'read','--model',str(a.model)]
rows=[]
def run(name,args,check):
 r=subprocess.run(args,capture_output=True,text=True,timeout=30)
 ok=bool(check(r));rows.append(dict(name=name,passed=ok,code=r.returncode,stdout=r.stdout,stderr=r.stderr));assert ok,(name,r.stdout,r.stderr)
run('plain_explicit_prediction',base+['--context',str(passage),'Who visited Zürich?'],lambda r:r.returncode==0 and 'Candidate answer (unverified): Élodie' in r.stdout and 'Source passage: Élodie visited Zürich in 2018' in r.stdout)
def valid_json(r):
 if r.returncode:return False
 d=json.loads(r.stdout);s=d['proposal'];return d['status']=='predicted' and d['reason']=='model_prediction_not_verified' and context.encode()[s['start']:s['end']].decode()==d['answer']
run('json_exact_utf8_provenance',base+['--context',str(passage),'--json','Who visited Zürich?'],valid_json)
run('explanation_is_not_probability',base+['--context',str(passage),'--explain','Who visited Zürich?'],lambda r:r.returncode==0 and '(not probabilities)' in r.stdout)
for key in ['id','title','answers','answer_start','frames']:
 source=a.output/(key+'.jsonl');source.write_text(json.dumps(dict(context=context,question='Who visited Zürich?',**{key:[]}))+'\n')
 run('reject_annotation_'+key,base+['--input',str(source)],lambda r:r.returncode!=0 and not r.stdout and 'unknown field' in r.stderr)
large=a.output/'large.txt';large.write_text('x'*65537)
run('bounded_context',base+['--context',str(large),'Who?'],lambda r:r.returncode!=0 and not r.stdout and 'limit' in r.stderr)
row=a.output/'large.jsonl';row.write_text('x'*(512*1024+1))
run('bounded_json_row',base+['--input',str(row)],lambda r:r.returncode!=0 and not r.stdout and 'limit' in r.stderr)
run('missing_question',base+['--context',str(passage)],lambda r:r.returncode!=0 and not r.stdout)
run('conflicting_modes',base+['--context',str(passage),'--input',str(row),'Who?'],lambda r:r.returncode!=0 and not r.stdout)
bad=a.output/'bad-model.json';bad.write_text('{}')
run('malformed_model',[str(a.binary),'read','--model',str(bad),'--context',str(passage),'Who?'],lambda r:r.returncode!=0 and not r.stdout)
model=json.loads(a.model.read_text());model['weights']=[3.4e38]*len(model['weights']);huge=a.output/'overflow-model.json';huge.write_text(json.dumps(model))
run('overflow_refuses',[str(a.binary),'read','--model',str(huge),'--context',str(passage),'--json','Who visited Zürich?'],lambda r:r.returncode==0 and json.loads(r.stdout)['status']=='refused' and json.loads(r.stdout)['reason']=='non_finite_model_score')
assert passage.read_text()==context
report=dict(total=len(rows),passed=sum(r['passed'] for r in rows),rows=rows,binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(a.model.read_bytes()).hexdigest(),scope='CLI contracts only; semantic failures remain in independent.json')
(a.output/'results.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='rows'}))
