"""Native conversation integration over the preinspection parent graph."""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--bundle',type=Path,required=True);p.add_argument('--model',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir();root=Path(__file__).parent;bundle=a.out/'authored-bundle';bundle.mkdir();shutil.copy2(root/'index.json',bundle/'index.json');shutil.copy2(a.bundle/'relation.json',bundle/'relation.json');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();manifest=dict(version=1,index_sha256=sha(bundle/'index.json'),relation_model_sha256=sha(bundle/'relation.json'),source='Independent preinspection authored conjunction graph',license='MIT OR Apache-2.0',scope='Finite authored development graph; no current or global truth');(bundle/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');records=[]
def invoke(lines,facts=bundle,session=None,production=False,human=False):
 cmd=[str(a.binary),'talk']
 if not human:cmd+=['--json']
 if facts:cmd+=['--facts-dir',str(facts)]
 if session:cmd+=['--session',str(session)]
 if production:cmd+=['--model',str(a.model)]
 proc=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,timeout=90);assert proc.returncode==0,proc.stderr
 return proc.stdout if human else [json.loads(l) for l in proc.stdout.splitlines()]
standalone={r['name']:r for r in json.loads((root/'candidate/results.json').read_text())};cases=json.loads((root/'cases.json').read_text());ran=[]
for case in cases:
 if '\n' in case['question']:continue # talk is a one-utterance-per-line protocol
 got=invoke(['Match facts: '+case['question']])[0];assert got['kind']=='fact_matches';assert got['provenance']['index_sha256']==manifest['index_sha256'];original=standalone[case['name']]
 if original.get('library_only'):assert got['status']=='refused'
 else:
  expected={k:v for k,v in original['actual'].items() if k!='index_sha256'};assert {k:got[k] for k in expected}==expected,(case,got)
 ran.append({'name':case['name'],'result':got})
(a.out/'replies.json').write_text(json.dumps(ran,ensure_ascii=False,indent=2)+'\n')
state_checks=0
requests=[
 'Match facts: Who was born in Harbor City and is a citizen of Maple Republic?',
 'Match facts: Who was born in Harbor City and works for Q123456?',
 'Match facts: Who was born in Springfield and is a citizen of Maple Republic?',
 'Match facts: Who was not born in Harbor City and is a citizen of Maple Republic?',
 'Match facts: My name is Changed',
 'Match facts Who was born in Harbor City and is a citizen of Maple Republic?',
 'Match facts: Who was born in Harbor City and is a citizen of Maple Republic? Move Mary to the garden.',
 'Match facts: Who was born in there and is a citizen of Maple Republic?',
 'Match facts: Who was born in Harbor\0City and is a citizen of Maple Republic?',
]
for production in (False,True):
 session=a.out/f'session-{production}.json';invoke(['My name is Mira','I like tea','Mary is in the kitchen.'],session=session,production=production)
 for request in requests:
  before=json.loads(session.read_text());reply=invoke([request],session=session,production=production)[0];after=json.loads(session.read_text());assert reply['kind']=='fact_matches';assert after['turn']==before['turn']+1 and after['last_slot'] is None and after['last_user_utterance']==request
  for k in set(before)-{'turn','last_slot','last_user_utterance'}:assert before[k]==after[k],(request,k)
  state_checks+=1
 replies=invoke(['What do I like?','What is my name?','Where is Mary?'],session=session,production=production);assert [r['kind'] for r in replies]==['recalled','recalled','story_answer'];assert 'tea' in replies[0]['text'] and 'Mira' in replies[1]['text'] and 'kitchen' in replies[2]['text'];state_checks+=3
 before=json.loads(session.read_text());reply=invoke(['Match facts: My name is Changed'],facts=None,session=session,production=production)[0];after=json.loads(session.read_text());assert reply['kind']=='refusal' and '--facts-dir' in reply['text'];assert before['name']==after['name'] and before['preferences']==after['preferences'] and before['stories']==after['stories'];state_checks+=1
# Case-insensitive prefix and unrelated personal names must remain distinct.
assert invoke(['  MATCH FACTS: Who died in Cobalt Bay and works for Cedar Lab?  '])[0]['status']=='matched'
assert invoke(['My name is Match facts'])[0]['kind']=='remembered'
human=invoke([requests[0]],human=True);assert '11' in human and 'first 10' in human and 'Q20' in human and 'https://www.wikidata.org/wiki/Q20#P19' in human and 'current status' in human;(a.out/'human.txt').write_text(human)
empty=invoke([requests[1]],human=True);assert 'No subjects match all conditions in the loaded snapshot' in empty
real=invoke(['Match facts: Who was born in Q220 and died in Q90?'],facts=a.bundle)[0];assert [m['subject'] for m in real['matches']]==['Q1029284'];(a.out/'real-reply.json').write_text(json.dumps(real,indent=2)+'\n')
assert sha(bundle/'index.json')==manifest['index_sha256'];report={'binary_sha256':sha(a.binary),'main_cases':len(ran),'multiline_library_and_cli_only':1,'state_and_recall_checks':state_checks,'production_intent_modes':[False,True],'prefix_and_personal_name_checks':2,'human_counts_witnesses_and_scope':True,'real_cached_edge_case':True,'all_passed':True};(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
