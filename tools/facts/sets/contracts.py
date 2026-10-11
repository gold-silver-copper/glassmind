#!/usr/bin/env python3
"""Authored product contracts; expected sets are derived only from fixture edges."""
import argparse, hashlib, json, subprocess
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument('--binary',required=True); p.add_argument('--output',type=Path,required=True); p.add_argument('--baseline',action='store_true'); a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=True); bundle=a.output/'bundle'; bundle.mkdir(exist_ok=True)
def write(name,value):
 b=json.dumps(value,ensure_ascii=False).encode(); (bundle/name).write_bytes(b); return hashlib.sha256(b).hexdigest()
def row(alias,claims=None):return dict(aliases=[alias],claims=claims or {})
entities={'Q1':row('Atlas',{'P1':['Q22','Q20','Q22','Q21']}),'Q2':row('Beacon'),'Q3':row('Cascade',{'P1':[f'Q{i}' for i in reversed(range(30,45))]+['Q30']}),'Q4':row('Delta'),'Q5':row('Étoile',{'P1':['Q21','Q20']}),'Q6':row('Twin'),'Q7':row('Twin'),'Q8':row('Empty')}
for i in range(60,74): entities[f'Q{i}']=row(f'Work {i}',{'P2':['Q4','Q4']+(['Q2'] if i<63 else [])})
index=write('index.json',dict(entities=entities))
# Minimal synthetic proposer fixture, not production weights or an answer table.
model=write('relation.json',dict(format_version=1,labels=['P1','R2'],training_rows=0,options=dict(epochs=1,char_weight=0,min_df=1),vocabulary={'w:created':dict(index=0,idf=1),'w:feature':dict(index=1,idf=1)},weights=[2,0,0,2],policy=dict(min_score=1,min_margin=1,wrong_cost=2,dev_rows=0,confidence_bins=[])))
write('manifest.json',dict(version=1,index_sha256=index,relation_model_sha256=model,source='authored development fixture',license='MIT OR Apache-2.0',scope='finite synthetic development graph'))
write('labels.json',dict(index_sha256=index,source='authored fixture',license='MIT OR Apache-2.0',labels={'Q1':'Display only name','Q20':'First creator'}))
contracts=json.loads(Path(__file__).with_name('contracts.json').read_text())
def run(lines,session=None,plain=False,with_bundle=True):
 cmd=[a.binary,'talk']+([] if plain else ['--json'])+(['--facts-dir',str(bundle)] if with_bundle else [])+(['--session',str(session)] if session else [])
 out=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,timeout=90,check=True).stdout
 return out if plain else [json.loads(x) for x in out.splitlines()]
def expected(subject,relation):
 if relation.startswith('P'):return sorted(set(entities[subject]['claims'].get(relation,[])))
 return sorted(s for s,e in entities.items() if subject in e['claims'].get('P'+relation[1:],[]))
checks=0
for text,subject,relation in contracts['positive']:
 ids=expected(subject,relation); assert len(ids)>1
 ordinary=text.split(': ',1)[1]
 legacy=run([ordinary])[0]; assert legacy['answers']==[] and legacy['refusal']; checks+=1
 reply=run([text])[0]
 if a.baseline:
  assert reply.get('kind')!='fact_set' and not reply.get('answers'); checks+=1; continue
 assert reply['kind']=='fact_set',reply
 assert reply['answers']==ids[:10] and reply['entity']==subject and reply['relation']==relation,reply
 assert reply['total_snapshot_matches']==len(ids) and reply['returned']==min(10,len(ids)) and reply['truncated']==(len(ids)>10)
 assert reply['provenance']['index_sha256']==index and reply['provenance']['relation_direction']==('inverse' if relation.startswith('R') else 'forward')
 assert {e['id'] for e in reply['entities']}==set(ids[:10]+[subject]); checks+=1
if not a.baseline:
 for text in contracts['negative']+['List answers to: Who created Display only name?']:
  r=run([text])[0]; assert r['kind']=='refusal' and not r['answers'],r; checks+=1
 text=contracts['positive'][0][0]
 plain=run([text],plain=True); assert 'label unavailable' in plain and 'snapshot' in plain and 'Property:P1' in plain; checks+=1
 r=run([text],with_bundle=False)[0]; assert r['kind']=='refusal'; checks+=1
 session=a.output/'session.json'
 if session.exists():session.unlink()
 lines=['My name is Mira','I like tea','Mary is in the kitchen.','What is 7 + 8?',text,'Where is Mary?','What is my name?']
 replies=run(lines,session); assert replies[3]['kind']=='calculated' and replies[4]['kind']=='fact_set' and replies[5]['kind']=='story_answer' and replies[6]['kind']=='recalled',replies
 saved=json.loads(session.read_text()); assert saved['name']['value']=='Mira'; checks+=4
 replies=run([text,'Actually, it is Changed','What is my name?','Where is Mary?'],session)
 assert replies[0]['kind']=='fact_set' and 'Mira' in replies[2]['text']; checks+=2
 # Separate persisted story check: correction may intentionally invalidate a story.
 replies=run(['/reset','Mary is in the kitchen.',text,'Where is Mary?'],session)
 assert replies[0]['kind']=='reset' and replies[2]['kind']=='fact_set' and replies[3]['kind']=='story_answer'; checks+=3
 replies=run([text,'Where is Mary?'],session); assert replies[1]['kind']=='story_answer'; checks+=1
 saved=json.loads(session.read_text()); assert saved['name'] is None; checks+=1
report=dict(checks=checks,baseline=a.baseline,binary_sha256=hashlib.sha256(Path(a.binary).read_bytes()).hexdigest(),fixture_index_sha256=index)
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n'); print(json.dumps(report))
