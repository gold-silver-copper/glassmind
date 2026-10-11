#!/usr/bin/env python3
"""Additional state invariants, using the already frozen synthetic fixture bundle."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--bundle',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
checks=0
for model in [False,True]:
 session=a.output/f'session-{model}.json'
 if session.exists():session.unlink()
 def run(lines):
  cmd=[a.binary,'talk','--json','--facts-dir',str(a.bundle),'--session',str(session)]
  if model:cmd+=['--model',str(a.bundle/'relation.json')]
  return [json.loads(x) for x in subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60).stdout.splitlines()]
 run(['My name is Mira','I like tea','Mary is in the kitchen.'])
 for text in ['List answers to: Who created Atlas?','List answers to: My name is Changed?','Show answers to: Who created Unknown?','List answers to: Who created Atlas? go to garden']:
  before=json.loads(session.read_text());reply=run([text])[0];after=json.loads(session.read_text())
  assert reply['kind'] in ['fact_set','refusal']
  assert after['turn']==before['turn']+1 and after['last_slot'] is None and after['last_user_utterance']==text
  for key in set(before)-{'turn','last_slot','last_user_utterance'}:assert after[key]==before[key],(text,key)
  checks+=1
 replies=run(['What do I like?','What is my name?','What is 7 + 8?','Where is Mary?'])
 assert [r['kind'] for r in replies]==['recalled','recalled','calculated','story_answer'],replies
 assert 'tea' in replies[0]['text'] and 'Mira' in replies[1]['text'];checks+=4
report=dict(checks=checks,intent_fixture_modes=[False,True]);(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
