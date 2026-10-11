"""Frozen prospective session contracts. Expected values never enter runtime input."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--model',type=Path);p.add_argument('--sample',action='store_true');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
checks=[];events=[]
def check(name,expected,actual):checks.append(dict(name=name,expected=expected,actual=actual,right=expected==actual))
rows=json.loads(Path(__file__).with_name('contracts.json').read_text())
for n,row in enumerate(rows[:6] if a.sample else rows):
 path=a.output/f'session{n}.json';path.unlink(missing_ok=True)
 def run(lines):
  cmd=[str(a.binary.resolve()),'talk','--json','--session',str(path)]+(['--model',str(a.model)] if a.model else [])
  r=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
  out=[json.loads(l) for l in r.stdout.splitlines()];assert len(out)==len(lines)
  state=json.loads(path.read_text());events.append(dict(session=n,inputs=lines,replies=out,state=state));return out,state
 def ck(k,e,v):check(f'{n}:{k}',e,v)
 setup=['My name is Hal','My favorite color is blue']
 if row.get('context')!='none':setup+=['Elin is in the kitchen.']
 if row.get('context')=='invalid':setup+=['Elin teleported to the attic.']
 setup+=['What is my name?']
 _,before=run(setup)
 if row['type']=='social':
  for repeat in range(2):
   replies,state=run([row['text']]);reply=replies[0]
   ck(f'social{repeat}', 'social' if row['text']=='Hello' else 'intent' if a.model else 'refusal',reply['kind'])
   ck('story preserved',before['stories'],state['stories']);ck('focus cleared',None,state['last_slot'])
   ck('exact utterance',row['text'],state['last_user_utterance']);ck('one turn',before['turn']+repeat+1,state['turn'])
   ck('name source',before['name'],state['name']);ck('preferences',before['preferences'],state['preferences'])
  replies,state=run(['What did I just say?']);ck('recall exact',True,row['text'] in replies[0]['text']);ck('recall source',before['turn']+2,replies[0].get('source_turn'))
  replies,state=run(["Actually, it's Nova"]);ck('correction no write',before['name'],state['name']);ck('correction not remembered',False,replies[0]['kind']=='remembered')
  replies,state=run(['/reset','Elin went to the garden.','Where is Elin?']);ck('reset then assertion','story_fact',replies[1]['kind']);ck('reset then answer','garden',replies[2].get('answer'));ck('reset count',2,state['turn'])
 elif row['type']=='contrast':
  replies,state=run([row['text']]);
  if row['kind']:ck('contrast kind',row['kind'],replies[0]['kind'])
  ck('invalidation',row['invalidated'],state['stories']['invalidated'])
  replies,state=run(['Where is Elin?']);ck('subsequent answer',row['answer'],replies[0].get('answer'))
 else:
  replies,state=run([row['text']]);expected=row['kind']
  if expected=='intent' and not a.model:expected='refusal'
  ck('priority kind',expected,replies[0]['kind'])
(a.output/'events.json').write_text(json.dumps(events,indent=2)+'\n')
report=dict(right=sum(c['right'] for c in checks),total=len(checks),sessions=len(rows[:6] if a.sample else rows),checks=checks)
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='checks'}))
