"""Fresh coordinator session/provenance contrasts; no changes to frozen worker tests."""
from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(exist_ok=False)
model=Path('/home/gsc/.cache/glassmind/models/everyday/intent.json')
cases=[
 ('social',['My name is Élodie','Hello'],"Actually, it's Rowan",'missing',['name'],'Élodie',1),
 ('utterance',['My name is Élodie','What did I just say?'],"Actually, it's Rowan",'missing',['name'],'Élodie',1),
 ('missing_slot',['My name is Élodie','What is my favorite flower?'],"Actually, it's Rowan",'missing',['name'],'Élodie',1),
 ('fact_list',['My name is Élodie','List answers to: Who created Atlas?'],"Actually, it's Rowan",'missing',['name'],'Élodie',1),
 ('story',['My name is Élodie','Mary is in the kitchen.'],"Actually, it's Rowan",'missing',['name'],'Élodie',1),
 ('calculation',['I prefer jasmine tea','What is 7 + 8?'],"Actually, it's oolong",'remembered',['preferences','general'],'oolong',3),
 ('explicit_restores',['My name is Élodie','Hello'],'Actually, my name is Rowan','remembered',['name'],'Rowan',3),
]
records=[]
for enabled in [False,True]:
 for name,setup,finish,kind,path,value,source in cases:
  stem=name+('-model' if enabled else '-plain');session=a.out/(stem+'.json');args=[str(a.binary.resolve()),'talk','--json','--session',str(session)]
  if enabled:args+=['--model',str(model)]
  rows=[]
  for index,lines in enumerate([setup,[finish]]):
   r=subprocess.run(args,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
   (a.out/(stem+f'-{index}.jsonl')).write_text(r.stdout)
   rows.extend(json.loads(l) for l in r.stdout.splitlines())
  state=json.loads(session.read_text());v=state
  for key in path:v=v[key]
  checks=dict(kind=rows[-1]['kind']==kind,value=v['value']==value,source=v['stated_on_turn']==source,turn=state['turn']==3,last_utterance=state['last_user_utterance']==finish)
  if kind=='missing':checks['no_correction']=state.get('last_correction') is None
  else:checks['correction_source']=state['last_correction']['previous']['stated_on_turn']==1 and state['last_correction']['replacement']['stated_on_turn']==3
  records.append(dict(case=stem,passed=all(checks.values()),checks=checks,reply=rows[-1],state=state))
report=dict(scope='Authored coordinator product session/provenance contrasts, not independent benchmark',binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),total=len(records),passed=sum(x['passed'] for x in records),records=records)
(a.out/'report.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='records'}))
