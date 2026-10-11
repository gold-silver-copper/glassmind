"""Native main-route replay of preimplementation contracts and immutable state."""
import argparse,copy,hashlib,json,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir();base=Path(__file__).parent;work=base.parents[1];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
casespath=base/'main-contracts.json';assert sha(casespath)=='42d3d482ea551980f33ea61f801510a7431fd9441fac3ace49ba1811a6e7736c';cases=json.loads(casespath.read_text())['cases'];bundle=a.out/'bundle';bundle.mkdir()
for name in ['index.json','temporal.json']:shutil.copy2(base/'candidate/year_interior'/name,bundle/name)
shutil.copy2(work/'factmatch-source/relation.json',bundle/'relation.json')
manifest=dict(version=1,index_sha256=sha(bundle/'index.json'),relation_model_sha256=sha(bundle/'relation.json'),temporal_sha256=sha(bundle/'temporal.json'),source='Parent authored temporal graph',license='MIT OR Apache-2.0',scope='Finite development graph; no current or global truth')
(bundle/'manifest.json').write_text(json.dumps(manifest))
def invoke(lines,facts=bundle,session=None,production=False,human=False,fail=False):
 cmd=[str(a.binary),'talk']
 if not human:cmd+=['--json']
 if facts:cmd+=['--facts-dir',str(facts)]
 if session:cmd+=['--session',str(session)]
 if production:cmd+=['--model','/home/gsc/.cache/glassmind/models/everyday/intent.json']
 proc=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,timeout=90)
 if fail:assert proc.returncode!=0 and not proc.stdout,(proc.stdout,proc.stderr);return proc.stderr
 assert proc.returncode==0,proc.stderr
 return proc.stdout if human else [json.loads(x) for x in proc.stdout.splitlines()]
rows=[];checks=0
for production in [False,True]:
 session=a.out/f'session-{production}.json';invoke(['My name is Mira','I like tea','Mary is in the kitchen.'],session=session,production=production)
 for c in cases:
  before=json.loads(session.read_text());v=invoke([c['line']],session=session,production=production)[0];after=json.loads(session.read_text())
  assert v['kind']=='fact_verification_at';assert v['status'] in ['unknown','refused'] if c['status']=='unknown_or_refused' else v['status']==c['status'],(c,v)
  assert after['turn']==before['turn']+1 and after['last_slot'] is None and after['last_user_utterance']==c['line']
  assert {k:v for k,v in before.items() if k not in ['turn','last_slot','last_user_utterance']}=={k:v for k,v in after.items() if k not in ['turn','last_slot','last_user_utterance']}
  rows.append(dict(name=c['name'],production=production,reply=v));checks+=1
 replies=invoke(['What do I like?','What is my name?','Where is Mary?'],session=session,production=production);assert ['tea' in replies[0]['text'],'Mira' in replies[1]['text'],'kitchen' in replies[2]['text']]==[True]*3;checks+=3
 invoke(['My name is Mira',cases[0]['line'],'Actually, Nora'],session=session,production=production)
 assert json.loads(session.read_text())['name']['value']=='Mira';checks+=1
 for facts in [None,bundle]:
  if facts:
   m=copy.deepcopy(manifest);del m['temporal_sha256'];(bundle/'manifest.json').write_text(json.dumps(m))
  before=json.loads(session.read_text());v=invoke([cases[0]['line']],facts=facts,session=session,production=production)[0];after=json.loads(session.read_text());assert v['kind']=='refusal';assert all(before[k]==after[k] for k in ['name','preferences','stories']);checks+=1
 (bundle/'manifest.json').write_text(json.dumps(manifest))
human=invoke([cases[0]['line']],human=True)
for part in ['2011-01-01','2010-12-31','2013-01-01','precision: 9','Q1$00000000-0000-4000-8000-000000000001','https://www.wikidata.org/wiki/Q1#P108','snapshot']:assert part in human,part
(a.out/'human.txt').write_text(human)
original=(bundle/'temporal.json').read_bytes();bad=[]
for kind in ['missing','hash_mismatch','index_binding','version']:
 m=copy.deepcopy(manifest)
 if kind=='missing':(bundle/'temporal.json').unlink()
 elif kind=='hash_mismatch':(bundle/'temporal.json').write_bytes(original+b' ')
 else:
  v=json.loads(original)
  if kind=='index_binding':v['index_sha256']='0'*64
  else:v['version']=2
  (bundle/'temporal.json').write_text(json.dumps(v));m['temporal_sha256']=sha(bundle/'temporal.json')
 (bundle/'manifest.json').write_text(json.dumps(m));error=invoke([cases[0]['line']],fail=True);bad.append(dict(kind=kind,error=error));(bundle/'temporal.json').write_bytes(original)
(bundle/'manifest.json').write_text(json.dumps(manifest));(a.out/'replies.json').write_text(json.dumps(rows,indent=2)+'\n');report=dict(binary_sha256=sha(a.binary),main_cases=len(rows),state_recall_checks=checks,invalid_bundle_cases=bad,human_witness=True,all_passed=True);(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
