from pathlib import Path
import json,subprocess,hashlib,shutil,sys
root=Path(__file__).resolve().parent;w=root.parents[2];binary=Path(sys.argv[1]).resolve();out=root/'main-contracts';out.mkdir(exist_ok=False);ind=w/'coordinator/factverify-independent';standalone=json.loads((ind/'alias-revision/results.json').read_text());records=[]
model=w/'coordinator/factsets-check/contracts/bundle/relation.json'
def invoke(lines,bundle=None,session=None,production=False):
 cmd=[str(binary),'talk','--json']
 if bundle:cmd+=['--facts-dir',str(bundle)]
 if session:cmd+=['--session',str(session)]
 if production:cmd+=['--model','/home/gsc/.cache/glassmind/models/everyday/intent.json']
 r=subprocess.run(cmd,input='\n'.join(lines)+'\n',capture_output=True,text=True,timeout=180,check=True);return r.stdout,[json.loads(l) for l in r.stdout.splitlines()]
for suite in ('primary','supplemental'):
 src=ind if suite=='primary' else ind/'supplemental';bundle=out/suite;bundle.mkdir();shutil.copyfile(src/'store.json',bundle/'index.json');shutil.copyfile(model,bundle/'relation.json')
 manifest={'version':1,'index_sha256':hashlib.sha256((bundle/'index.json').read_bytes()).hexdigest(),'relation_model_sha256':hashlib.sha256(model.read_bytes()).hexdigest(),'source':'independent authored verification graph','license':'MIT OR Apache-2.0','scope':'synthetic graph for native integration contracts'};(bundle/'manifest.json').write_text(json.dumps(manifest))
 expected=[x for x in standalone if x['suite']==suite and '\n' not in x['case']['text']];lines=['Check fact: '+x['case']['text'] for x in expected];raw,replies=invoke(lines,bundle);(bundle/'replies.jsonl').write_text(raw);assert len(replies)==len(expected)
 for e,a in zip(expected,replies,strict=True):
  subset={k:a[k] for k in ('status','witness','scope','reason') if k in a};assert subset==e['actual'],(e,a)
  assert a['kind']=='fact_verification' and a['provenance']['index_sha256']==manifest['index_sha256'];records.append({'suite':suite,'case':e['case']['name'],'matches_standalone':True,'matches_original_expectation':e['pass']})
# Every explicitly prefixed request, including malformed and mixed ones, preserves
# factual/personal/story state apart from the turn log and clearing follow-up focus.
bundle=out/'primary';state_checks=0
for production in (False,True):
 session=out/f'session-{production}.json';invoke(['My name is Mira','I like tea','Mary is in the kitchen.'],bundle,session,production)
 for line in ['Check fact: Was Mara Finch born in North Haven?','Check fact: Was Mara Finch born in South Haven?','Check fact: My name is Changed','Check fact: Was Mara Finch born in North Haven? Move Mary to the garden.','Check fact Was Mara Finch born in North Haven?','Check fact: Was she born in North Haven?']:
  before=json.loads(session.read_text());_,reply=invoke([line],bundle,session,production);after=json.loads(session.read_text());assert reply[0]['kind']=='fact_verification'
  assert after['turn']==before['turn']+1 and after['last_slot'] is None and after['last_user_utterance']==line
  for k in set(before)-{'turn','last_slot','last_user_utterance'}:assert before[k]==after[k],(line,k)
  state_checks+=1
 _,reply=invoke(['What do I like?','What is my name?','Where is Mary?'],bundle,session,production);assert [r['kind'] for r in reply]==['recalled','recalled','story_answer'];assert 'tea' in reply[0]['text'] and 'Mira' in reply[1]['text'] and 'kitchen' in reply[2]['text'];state_checks+=3
_,disabled=invoke(['Check fact: Was Mara Finch born in North Haven?']);assert disabled[0]['kind']=='refusal' and '--facts-dir' in disabled[0]['text']
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'native_cases':len(records),'library_only_multiline_cases':1,'all_standalone_results_and_witnesses_identical':True,'original_expectations_matched':sum(x['matches_original_expectation'] for x in records),'session_invariants':state_checks,'facts_disabled_explicit_refusal':True,'cases':records};(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'},indent=2))
