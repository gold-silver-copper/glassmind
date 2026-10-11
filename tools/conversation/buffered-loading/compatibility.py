from pathlib import Path
import hashlib,json,subprocess
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator');out=r/'model-load-review/compatibility';out.mkdir(exist_ok=False);model=Path('/home/gsc/.cache/glassmind/models/everyday/intent.json');a=r/'routeauthority-review/glassmind-integrated';b=r/'buffered-route-review/glassmind-integrated';records=[]
for label,content in [('empty',b''),('truncated',b'{"format_version":1'),('shape',b'[]'),('unicode',b'{"bad":"\xff"}')]:
 f=out/(label+'.json');f.write_bytes(content)
 for kind in ['model','session']:
  results=[]
  for binary in [a,b]:
   v=subprocess.run([str(binary),'talk','--json','--'+kind,str(f)],input='Hello\n',text=True,capture_output=True,timeout=10);results.append(dict(code=v.returncode,stdout=v.stdout,stderr=v.stderr))
  assert results[0]==results[1] and results[0]['code']!=0;assert f.read_bytes()==content
  records.append(dict(case=label,kind=kind,identical=True,result=results[0]))
for binary in [a,b]:
 v=subprocess.run([str(binary),'talk','--json','--model',str(out/'missing.json')],input='Hello\n',text=True,capture_output=True,timeout=10)
 if binary==a:first=dict(code=v.returncode,stdout=v.stdout,stderr=v.stderr)
 else:assert first==dict(code=v.returncode,stdout=v.stdout,stderr=v.stderr) and v.returncode!=0
records.append(dict(case='missing',kind='model',identical=True,result=first))
invalid=json.loads(model.read_text());invalid['format_version']=4294967295;bad=out/'invalid-version.json';bad.write_text(json.dumps(invalid));del invalid
for binary in [a,b]:
 v=subprocess.run([str(binary),'talk','--json','--model',str(bad)],input='Hello\n',text=True,capture_output=True,timeout=10)
 if binary==a:first=dict(code=v.returncode,stdout=v.stdout,stderr=v.stderr)
 else:assert first==dict(code=v.returncode,stdout=v.stdout,stderr=v.stderr) and v.returncode!=0
records.append(dict(case='invalid-version',kind='model',identical=True,result=first))
seed=(r/'routeauthority-review/boundary-full-plain/session0.json').read_bytes();answers=[];states=[]
for name,binary in [('baseline',a),('buffered',b)]:
 path=out/(name+'-session.json');path.write_bytes(seed)
 v=subprocess.run([str(binary),'talk','--json','--model',str(model),'--session',str(path)],input='Hello\nWhat is my name?\nWhere is Mira?\n',text=True,capture_output=True,check=True,timeout=10);answers.append(v.stdout);states.append(path.read_bytes())
assert answers[0]==answers[1] and states[0]==states[1]
report=dict(baseline_binary_sha256=hashlib.sha256(a.read_bytes()).hexdigest(),candidate_binary_sha256=hashlib.sha256(b.read_bytes()).hexdigest(),rejection_cases=len(records),all_rejections_identical=True,persisted_state_and_replies_identical=True,loaded_session_sha256=hashlib.sha256(seed).hexdigest(),cases=records)
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
