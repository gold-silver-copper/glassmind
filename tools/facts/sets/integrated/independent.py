from pathlib import Path
import hashlib,json,shutil,subprocess
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');OUT=W/'coordinator/factsets-check/independent';OUT.mkdir()
BINARY=Path('/home/gsc/code/glassmind/target/release/glassmind');MODEL=Path('/home/gsc/.cache/glassmind/models/everyday/intent.json');BUNDLE=OUT/'bundle';shutil.copytree(W/'factsets-run/tmp/sets/contracts/bundle',BUNDLE)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def call(lines,session=None):
 args=[str(BINARY),'talk','--json','--model',str(MODEL),'--facts-dir',str(BUNDLE)]
 if session:args+=['--session',str(session)]
 p=subprocess.run(args,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
 return [json.loads(s) for s in p.stdout.splitlines()]
records=[]
for n in [10,11]:
 idx=json.loads((BUNDLE/'index.json').read_text());values=['Q'+str(i) for i in range(80,80+n)];idx['entities']['Q1']['claims']['P1']=list(reversed(values))+values[:3]
 (BUNDLE/'index.json').write_text(json.dumps(idx));digest=sha(BUNDLE/'index.json')
 for name in ['manifest.json','labels.json']:
  x=json.loads((BUNDLE/name).read_text());x['index_sha256']=digest;(BUNDLE/name).write_text(json.dumps(x))
 r=call(['sHoW AnSwErS To: Who created Atlas?'])[0]
 assert r['kind']=='fact_set' and r['answers']==sorted(values)[:10]
 assert r['returned']==10 and r['total_snapshot_matches']==n and r['truncated']==(n>10)
 records.append({'case':'exact_display_boundary_'+str(n),'reply':r,'passed':True})
session=OUT/'session.json';call(['My name is Mira','I like tea','Mary is in the kitchen.'],session)
for text in ['List answers to: Who created Atlas?','List answers to: My name is Changed?','Show answers to: Who created Unknown?','List answers to: Who\tcreated Atlas?']:
 before=json.loads(session.read_text());r=call([text],session)[0];after=json.loads(session.read_text());assert r['kind'] in ['fact_set','refusal']
 assert after['turn']==before['turn']+1 and after['last_slot'] is None and after['last_user_utterance']==text
 assert {k:v for k,v in before.items() if k not in ['turn','last_slot','last_user_utterance']}=={k:v for k,v in after.items() if k not in ['turn','last_slot','last_user_utterance']}
 records.append({'case':'production_intent_state_'+str(len(records)),'question':text,'kind':r['kind'],'passed':True})
r=call(['What do I like?','What is my name?','What is 7 + 8?','Where is Mary?'],session)
assert [x['kind'] for x in r]==['recalled','recalled','calculated','story_answer'];assert 'tea' in r[0]['text'] and 'Mira' in r[1]['text']
records.append({'case':'production_intent_recalls_and_reasoning','replies':r,'passed':True})
report={'checks':len(records),'production_intent_model':{'path':str(MODEL),'bytes':MODEL.stat().st_size,'sha256':sha(MODEL)},'binary_sha256':sha(BINARY),'records':records,'scope':'Fresh display-boundary and persisted-state probes with production conversation model; relation model remains the independent authored fixture, not a factual accuracy evaluation.'}
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'checks':len(records),'all_passed':all(x['passed'] for x in records),'production_intent_sha256':sha(MODEL)}))
