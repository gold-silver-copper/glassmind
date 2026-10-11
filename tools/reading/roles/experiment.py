#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""One preregistered ranker experiment; official scorer imported without edits."""
import argparse, collections, hashlib, importlib.util, json, os, pathlib, shutil
HERE=pathlib.Path(__file__).resolve().parent
RUN=pathlib.Path(os.environ['ROLES_RUN']).resolve()
BASE=pathlib.Path(os.environ['ROLES_BASE']).resolve()
REPORT=HERE/'reports'
os.environ['QA_ARTIFACTS_DIR']=str(RUN)
os.environ['QA_REPORT_DIR']=str(REPORT)
spec=importlib.util.spec_from_file_location('original_qa',HERE.parent/'qa/run.py')
qa=importlib.util.module_from_spec(spec); spec.loader.exec_module(qa)
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p): return json.loads(p.read_text())
def predictions(p): return [json.loads(l) for l in p.read_text().splitlines()]
def write(p,v): qa.write(p,v)
def prepare():
 RUN.mkdir(parents=True,exist_ok=True); REPORT.mkdir(exist_ok=True)
 assert sha(BASE/'model.json')=='b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5'
 # Recreate TRAIN selection independently, prove exact original rows and hashes.
 qa.prepare()
 for name in ['fit.input.jsonl','fit.train.jsonl','cal.input.jsonl','cal.labels.json']:
  assert (RUN/name).read_bytes()==(BASE/name).read_bytes(),name
 assert read(REPORT/'selection.json')==read(BASE/'selection.json')
 write(REPORT/'baseline.json',read(BASE/'full.json'))
 write(REPORT/'train-inspection.json',predictions(BASE/'fit.train.jsonl')[:14])
 probes=[]
 def add(name,c,q,expect): probes.append(dict(name=name,context=c,question=q,expected=expect))
 for c,who,obj in [('Nora thanked Felix.','Nora','Felix'),('Felix thanked Nora.','Felix','Nora')]:
  add('role-subject-'+who,c,'Who thanked '+obj+'?',who)
  add('role-object-'+obj,c,'Who did '+who+' thank?',obj)
 add('role-reversed-unsupported-subject','Felix thanked Nora.','Who thanked Felix?',None)
 add('role-reversed-unsupported-object','Felix thanked Nora.','Who did Nora thank?',None)
 for pred,expect in [('demolished','2008'),('painted',None),('never demolished',None)]:
  add('predicate-'+pred,'Nora '+pred+' the bridge in 2008.','When did Nora demolish the bridge?',expect)
 add('negative-positive','Nora never demolished the bridge in 2008.','When did Nora never demolish the bridge?','2008')
 add('kind-person','Mira met Soren in Oslo.','Who did Mira meet?','Soren')
 add('kind-place','Mira met Soren in Oslo.','Where did Mira meet Soren?','Oslo')
 add('kind-number','Mira carried three crates.','How many crates did Mira carry?','three')
 add('kind-unsupported','Mira carried crates.','How many crates did Mira carry?',None)
 add('unicode-person','Élodie visited Zürich.','Who visited Zürich?','Élodie')
 add('unicode-place','Élodie visited Zürich.','Where did Élodie visit?','Zürich')
 add('unsupported','Lena owns a bicycle.','When did Lena buy the bicycle?',None)
 add('supported','Lena bought a bicycle in 2012.','When did Lena buy the bicycle?','2012')
 write(HERE/'probes.json',probes)
 qa.lines(RUN/'probes.input.jsonl',[dict(context=p['context'],question=p['question']) for p in probes])
 write(REPORT/'preregister-hashes.json',{str(p):sha(p) for p in [HERE/'PREREGISTER.md',HERE/'probes.json',HERE/'experiment.py']})
def freeze(binary):
 qa.calibrate(binary)
 assert (RUN/'model.json').stat().st_size<=2*1024*1024
 write(REPORT/'freeze.json',dict(model_sha256=sha(RUN/'model.json'),binary_sha256=sha(binary),model_bytes=(RUN/'model.json').stat().st_size))
def frozen(binary):
 f=read(REPORT/'freeze.json'); assert sha(binary)==f['binary_sha256']; assert sha(RUN/'model.json')==f['model_sha256']
def transitions(rs,bs,cs,name):
 counts={'raw':collections.Counter(),'selected':collections.Counter()}; allrows=[]
 for r,b,c in zip(rs,bs,cs):
  out=dict(id=r['id'])
  for kind in counts:
   bt=qa.proposal(r,b) if kind=='raw' else b['answer']; ct=qa.proposal(r,c) if kind=='raw' else c['answer']
   be=max(qa.SCORE['compute_exact'](a['text'],bt) for a in r['answers']); ce=max(qa.SCORE['compute_exact'](a['text'],ct) for a in r['answers'])
   bstate='right' if be else ('refused' if kind=='selected' and b['status']=='refused' else 'wrong')
   cstate='right' if ce else ('refused' if kind=='selected' and c['status']=='refused' else 'wrong')
   tag=bstate+'->'+cstate; counts[kind][tag]+=1
   out[kind]=dict(before=bt,after=ct,transition=tag,changed=bt!=ct)
  allrows.append(out)
 write(RUN/(name+'.transitions.json'),allrows)
 return {k:dict(v) for k,v in counts.items()}
def evaluate(binary,size):
 frozen(binary)
 rs=sorted(qa.rows(qa.load('dev-v1.1.json')['data']),key=qa.order)
 assert len(rs)==10570 and len({r['id'] for r in rs})==10570
 original_inputs=predictions(BASE/'full.input.jsonl')
 assert original_inputs==[dict(context=r['context'],question=r['question']) for r in rs]
 bs=predictions(BASE/'full.candidate.outputs.jsonl'); assert len(bs)==10570
 if size=='sample': rs=rs[:200]; bs=bs[:200]
 qa.lines(RUN/(size+'.input.jsonl'),[dict(context=r['context'],question=r['question']) for r in rs])
 cs=qa.infer(binary,RUN/(size+'.input.jsonl'),RUN/'model.json',RUN/(size+'.outputs.jsonl'))
 assert len(cs)==len(rs)==(200 if size=='sample' else 10570)
 b=qa.metrics(rs,bs,size+'.baseline'); c=qa.metrics(rs,cs,size+'.candidate')
 tr=transitions(rs,bs,cs,size)
 raw_right=tr['raw'].get('right->right',0)+tr['raw'].get('wrong->right',0)
 result=dict(baseline=b,candidate=c,transitions=tr,raw_right=raw_right,selection='SHA256(questionID) ascending',passes_frozen_gate=(raw_right>=2638 and c['raw']['f1']>=35.518425 and c['right']>=815 and c['precision'] is not None and c['precision']>=.53) if size=='full' else None)
 if size=='full':
  replay=RUN/'v1.replay.outputs.jsonl'; qa.infer(binary,BASE/'full.input.jsonl',BASE/'model.json',replay)
  assert replay.read_bytes()==(BASE/'full.candidate.outputs.jsonl').read_bytes()
  result['legacy_byte_identical_all10570']=True
 write(REPORT/(size+'.json'),result); print(json.dumps(result,indent=2))
def probes(binary):
 frozen(binary); cases=read(HERE/'probes.json')
 for name,model in [('baseline',BASE/'model.json'),('candidate',RUN/'model.json')]:
  ps=qa.infer(binary,RUN/'probes.input.jsonl',model,RUN/(name+'.probes.outputs.jsonl'))
  assert len(ps)==len(cases)
  for c,p in zip(cases,ps):
   raw=qa.proposal(c,p)
   c[name]=dict(prediction=p,raw=raw,raw_correct=None if c['expected'] is None else raw==c['expected'],selected_behavior_correct=(p['status']=='refused' if c['expected'] is None else p['answer']==c['expected']))
 write(REPORT/'probes.json',cases)
 print(json.dumps({name:dict(passed=sum(c[name]['selected_behavior_correct'] for c in cases),total=len(cases),failures=[c['name'] for c in cases if not c[name]['selected_behavior_correct']]) for name in ['baseline','candidate']},indent=2))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('stage',choices=['prepare','freeze','sample','full','probes']);p.add_argument('--binary',type=pathlib.Path);a=p.parse_args()
 if a.stage=='prepare': prepare()
 elif a.stage=='freeze': freeze(a.binary.resolve())
 elif a.stage=='probes': probes(a.binary.resolve())
 else: evaluate(a.binary.resolve(),a.stage)
