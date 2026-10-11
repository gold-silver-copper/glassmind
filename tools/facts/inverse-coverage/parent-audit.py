from pathlib import Path
import hashlib,json,importlib.util,copy,re,subprocess
from collections import Counter
from datetime import datetime
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');R=W/'inversefacts-run/tmp/inverse-coverage';ROOT=Path('/home/gsc/code/glassmind');OWN=ROOT/'tools/facts/inverse-coverage';BASE=W/'trainindex-run/tmp/trainindex';OUT=W/'coordinator/inversefacts-review'
def load(p):return json.loads(p.read_bytes())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
checked={}
def inventory(obj):
 if isinstance(obj,dict):
  if {'path','sha256','bytes'}<=obj.keys():
   p=Path(obj['path']);assert p.is_absolute() and ('/codex/' not in str(p)) and p.name!='auth.json'
   assert str(p).startswith((str(W)+'/',str(ROOT)+'/','/home/gsc/.cache/glassmind/data/')),str(p)
   if p.exists(): raw=p.read_bytes()
   else:
    relative=p.relative_to(W/'inversefacts')
    if str(relative).startswith('tools/facts/inverse-coverage/'):
     raw=(W/'inversefacts-run/submitted'/relative.relative_to('tools/facts/inverse-coverage')).read_bytes()
    else:
     raw=subprocess.check_output(['git','show','b569ac0eb0bab5a8b9d39cda904c816e77c08e63:'+str(relative)],cwd=ROOT)
   assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['sha256'],str(p)
   checked[str(p)]=obj['sha256']
  for v in obj.values():inventory(v)
 elif isinstance(obj,list):
  for v in obj:inventory(v)
for f in ['base-verification.json','candidate-freeze.json','fetch.json','merge.json','summary.json']:
 inventory(load(R/f))
base=load(BASE/'scope1000/index.json');candidate=load(R/'candidate-index.json');sel=load(OWN/'selection.json');ledger=load(R/'fetch.json')
assert sha(OWN/'selection.json')=='e83ebf84d3574b2fcea42254bfb65fc4077a5d6250e9317b65f35c8933af0a45'
raw=Path('/home/gsc/.cache/glassmind/data/simplequestions/train.tsv').read_bytes();assert hashlib.sha256(raw).hexdigest()==sel['train_sha256'];train=raw.splitlines();counts=Counter()
for i,line in enumerate(train[:len(train)*9//10]):
 if i%10==0:continue
 a,b,_=line.split(b'\t',2);a,b=a.decode(),b.decode()
 if a in base['entities'] and re.fullmatch(r'Q[0-9]+',a) and re.fullmatch(r'R[0-9]+',b):counts[a,b]+=1
selected=sorted(counts,key=lambda p:(-counts[p],hashlib.sha256((p[0]+'\t'+p[1]).encode()).hexdigest()))[:100]
assert [(x['subject'],x['relation']) for x in sel['pairs']]==selected
assert [x['count'] for x in sel['pairs']]==[counts[p] for p in selected]
expected=copy.deepcopy(base);edges=set();received=0;starts=[];statuses=Counter()
for request in ledger['requests']:
 starts.append(request['start_elapsed_seconds']);pair=sel['pairs'][request['pair_index']]
 if 'response' in request:received+=request['response']['bytes']
 if not request['success']:
  statuses['missing']+=1;continue
 assert request['http_status']==200
 rows=load(Path(request['response']['path']))['results']['bindings'];assert len(rows)==101
 ids=[]
 for r in rows:
  subject=r['subject']['value'].rsplit('/',1)[1];ids.append(subject)
  for field in ['subject','target','property','rank','statement']:assert r[field]['type']=='uri'
  assert re.fullmatch(r'Q[0-9]+',subject)
  prop='P'+pair['relation'][1:];target=pair['subject']
  assert r['target']['value']=='http://www.wikidata.org/entity/'+target
  assert r['property']['value']=='http://www.wikidata.org/entity/'+prop
  assert r['rank']['value'] in ['http://wikiba.se/ontology#NormalRank','http://wikiba.se/ontology#PreferredRank']
  assert r['statement']['value'].lower().startswith('http://www.wikidata.org/entity/statement/'+subject.lower()+'-')
  edges.add((subject,prop,target))
 assert len(set(ids))==101;statuses['truncated']+=1
for subject,prop,target in sorted(edges):
 e=expected['entities'].setdefault(subject,dict(aliases=[],claims={}))
 values=e['claims'].setdefault(prop,[])
 if target not in values:values.append(target)
for entity in expected['entities'].values():
 for p in entity['claims']:entity['claims'][p]=sorted(entity['claims'][p])
actual=copy.deepcopy(candidate)
for entity in actual['entities'].values():
 for p in entity['claims']:entity['claims'][p]=sorted(entity['claims'][p])
assert actual==expected
assert all(candidate['entities'][q]==e for q,e in base['entities'].items())
assert all(not e['aliases'] for q,e in candidate['entities'].items() if q not in base['entities'])
assert len(starts)<=110 and all(b-a>=5-1e-5 for a,b in zip(starts,starts[1:]));assert received==ledger['new_decompressed_bytes']<=20*1024*1024
seconds=(datetime.fromisoformat(ledger['finished_utc'])-datetime.fromisoformat(ledger['started_utc'])).total_seconds();assert seconds<=600
spec=importlib.util.spec_from_file_location('joint',ROOT/'tools/facts/joint-audit.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
valid=Path('/home/gsc/.cache/glassmind/data/simplequestions/valid.tsv');labels=[s.split('\t',3) for s in valid.read_text().splitlines()];evaluated=[]
for split,n in [('sample',200),('full',2821)]:
 directory=R/split;rows=labels[:n];assert (directory/'questions.txt').read_text()==''.join(r[3]+'\n' for r in rows)
 assert (directory/'baseline-standalone.jsonl').read_bytes()==(directory/'candidate-standalone.jsonl').read_bytes()
 for mode in ['baseline','candidate']:
  predictions=[json.loads(l) for l in (directory/(mode+'-standalone.jsonl')).read_text().splitlines()];report=audit.audit(rows,predictions)
  assert report==load(directory/(mode+'-joint.json'))
  official=load(directory/(mode+'-score.json'));assert all(official[k]==report['answer_only'][k] for k in ['right','refused','wrong'])
  talk=[json.loads(l) for l in (directory/(mode+'-talk.jsonl')).read_text().splitlines()];assert len(talk)==2*n
  assert all(r['kind']=='reset' for r in talk[::2]);assert all(p['answers']==t.get('answers',[]) for p,t in zip(predictions,talk[1::2],strict=True))
 assert (directory/'relocated-talk.jsonl').read_bytes()==(directory/'candidate-talk.jsonl').read_bytes()
 assert load(directory/'report.json')['accepted'] is False
 evaluated.append({'split':split,'rows':n,'answer_only':report['answer_only'],'joint':report['joint_entity_relation_answer']})
result={'decision':'REJECTED partial candidate; no inference repeated','verified_artifacts':len(checked),'frozen_artifacts':len(load(R/'candidate-freeze.json')['artifacts']),'selected_pairs':100,'selected_train_rows':sum(counts[p] for p in selected),'requests':len(starts),'statuses':dict(statuses),'not_requested':100-len(starts),'received_bytes':received,'network_seconds':seconds,'independent_edges':len(edges),'new_nodes':len(candidate['entities'])-len(base['entities']),'all_original_records_unchanged':True,'candidate_graph_equals_independent_response_edge_merge':True,'evaluation':evaluated,'standalone_before_after_byte_identical':True,'all_talk_answer_ids_equal':True,'relocation_byte_identical':True,'tests_passed':7,'raw_artifact_identities':checked,'scope':'Frozen data selection and acquired edges validated; all queries truncated and primary graph incomplete. No global uniqueness, new benchmark gain, or complete acquisition claim.'}
(OUT/'parent-review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='raw_artifact_identities'},indent=2))
