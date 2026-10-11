"""Replay preimplementation main contracts with per-case immutable facts bundles."""
from pathlib import Path
import argparse,copy,hashlib,json,shutil,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir();base=Path(__file__).parent;work=base.parents[1];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(base/'main-contracts.json')=='2661ad0b9cac9e903adea7b9fd9856e5800debf98610f5bc9638ff8fd1f3f8ee';cases=json.loads((base/'main-contracts.json').read_text())['cases'];bundles={};manifests={}
for name in sorted({c['case'] for c in cases}):
 bundle=a.out/'bundles'/name;bundle.mkdir(parents=True)
 for item in ['index.json','temporal.json']:shutil.copy2(base/'inputs'/name/item,bundle/item)
 (bundle/'relation.json').symlink_to(work/'factmatch-source/relation.json');m=dict(version=1,index_sha256=sha(bundle/'index.json'),relation_model_sha256=sha(bundle/'relation.json'),temporal_sha256=sha(bundle/'temporal.json'),source='Parent authored temporal conjunction graph',license='MIT OR Apache-2.0',scope='Finite development graph; no global/current truth');(bundle/'manifest.json').write_text(json.dumps(m));bundles[name]=bundle;manifests[name]=m
def invoke(lines,bundle=None,session=None,production=False,human=False):
 cmd=[str(a.binary),'talk']
 if not human:cmd+=['--json']
 if bundle:cmd+=['--facts-dir',str(bundle)]
 if session:cmd+=['--session',str(session)]
 if production:cmd+=['--model','/home/gsc/.cache/glassmind/models/everyday/intent.json']
 r=subprocess.run(cmd,input='\n'.join(lines)+'\n',capture_output=True,text=True,timeout=90);assert r.returncode==0,r.stderr
 return r.stdout if human else [json.loads(s) for s in r.stdout.splitlines()]
def state(before,after,line):
 assert after['turn']==before['turn']+1 and after['last_user_utterance']==line and after['last_slot'] is None
 without=lambda s:{k:v for k,v in s.items() if k not in ['turn','last_user_utterance','last_slot']}
 assert without(before)==without(after)
rows=[];checks=0
for production in [False,True]:
 session=a.out/f'session-{production}.json';invoke(['My name is Mira','I like tea','Mary is in the kitchen.'],session=session,production=production)
 for c in cases:
  before=json.loads(session.read_text());v=invoke([c['line']],bundle=bundles[c['case']],session=session,production=production)[0];state(before,json.loads(session.read_text()),c['line']);assert v['kind']=='fact_matches_at' and v['status']==c['expected_status'],(c,v)
  if v['status']=='matched':
   expect=c['expected_subjects'];assert [x['subject'] for x in v['matches']]==expect[:10] and v['total_recorded_interval_matches']==len(expect) and v['returned']==min(10,len(expect)) and v['truncated']==(len(expect)>10)
  if 'name' not in c:
   expected=json.loads((base/'candidate'/(c['case']+'.json')).read_text());assert {k:v for k,v in v.items() if k not in ['kind','text','provenance']}==expected
  rows.append(dict(case=c.get('name',c['case']),production=production,reply=v));checks+=1
 replies=invoke(['What do I like?','What is my name?','Where is Mary?'],session=session,production=production);assert 'tea' in replies[0]['text'] and 'Mira' in replies[1]['text'] and 'kitchen' in replies[2]['text'];checks+=3
 bundle=bundles[cases[0]['case']];invoke(['My name is Mira',cases[0]['line'],'Actually, Nora'],bundle=bundle,session=session,production=production);assert json.loads(session.read_text())['name']['value']=='Mira';checks+=1
 for facts in [None,bundle]:
  if facts:
   m=copy.deepcopy(manifests[cases[0]['case']]);del m['temporal_sha256'];(bundle/'manifest.json').write_text(json.dumps(m))
  before=json.loads(session.read_text());v=invoke([cases[0]['line']],bundle=facts,session=session,production=production)[0];state(before,json.loads(session.read_text()),cases[0]['line']);assert v['kind']=='refusal';assert ('temporal.json' if facts else '--facts-dir') in v['text'];checks+=1
 (bundle/'manifest.json').write_text(json.dumps(manifests[cases[0]['case']]))
human_cases=[]
for c in [cases[0],cases[1],cases[2]]:
 human=invoke([c['line']],bundle=bundles[c['case']],human=True);expected=json.loads((base/'candidate'/(c['case']+'.json')).read_text());assert expected['date'] in human and str(expected['total_recorded_interval_matches']) in human and 'snapshot' in human and 'zero matches is not global absence' in human
 if expected['truncated']:assert 'first 10' in human
 for m in expected['matches']:
  assert m['subject'] in human
  for w in m['witnesses']:
   for value in [w['claim_id'],w['source_url'],w['start']['latest'],w['end']['earliest']]:assert value in human,value
 (a.out/(c['case']+'-human.txt')).write_text(human);human_cases.append(c['case'])
(a.out/'replies.json').write_text(json.dumps(rows,indent=2)+'\n');report=dict(binary_sha256=sha(a.binary),main_cases=len(rows),state_recall_checks=checks,human_cases=human_cases,all_passed=True);(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
