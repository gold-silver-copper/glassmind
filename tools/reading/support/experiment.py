#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Frozen one-candidate experiment. Scoring imports unchanged official functions."""
import argparse, collections, hashlib, importlib.util, json, math, os, pathlib, subprocess
ROOT=pathlib.Path(__file__).resolve().parents[3]
RUN=pathlib.Path(os.environ.get('SUPPORT_RUN',ROOT.parent/'qasupport-run/support'))
BASE=RUN.parent/'baseline'
spec=importlib.util.spec_from_file_location('qa_run',ROOT/'tools/reading/qa/run.py'); qa=importlib.util.module_from_spec(spec); spec.loader.exec_module(qa)
qa.ART=RUN
BINARY=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release/reading-support'
def read(p):return [json.loads(l) for l in p.read_text().splitlines()]
def infer(name,support=False,features=True):
 out=RUN/(name+('.supported' if support else '.features')+'.jsonl')
 cmd=[str(BINARY),'--input',str(RUN/(name+'.input.jsonl')),'--model',str(BASE/'model.json')]
 if support:cmd+=['--support',str(RUN/'support-model.json')]
 if features:cmd+=['--features']
 with out.open('w') as f:subprocess.run(cmd,stdout=f,check=True,timeout=590)
 return read(out)
def label(rs,ps):return [max(qa.SCORE['compute_exact'](a['text'],p['prediction']['answer']) for a in r['answers']) for r,p in zip(rs,ps)]
def threshold(scores,ys):
 ranked=sorted(zip(scores,ys),reverse=True);right=0;best=None
 for i,(score,y) in enumerate(ranked):
  right+=y
  if i+1<len(ranked) and ranked[i+1][0]==score:continue
  if i+1>=30 and right/(i+1)>=.9:best=dict(threshold=score,accepted=i+1,right=right,wrong=i+1-right,precision=right/(i+1))
 return best

def train():
 fs=infer('fit');cs=infer('cal')
 fr=json.loads((RUN/'fit.labels.json').read_text());cr=json.loads((RUN/'cal.labels.json').read_text())
 fy=label(fr,fs);cy=label(cr,cs)
 take=[i for i,p in enumerate(fs) if p['prediction']['status']=='predicted']
 x=[fs[i]['features'] for i in take];y=[fy[i] for i in take]; d=len(x[0]);assert d==48
 cuts=[]
 for j in range(d):
  values=sorted(set(r[j] for r in x)); cuts.append(sorted(set(values[min(len(values)-1,len(values)*k//16)] for k in range(1,16))))
 bias=sum(y)/len(y);pred=[bias]*len(y);trees=[]
 def tree(indices,residual,depth=0):
  total=sum(residual[i] for i in indices);mean=total/len(indices)
  if depth==2 or len(indices)<30:return {'value':.1*mean}
  best=None
  for j in range(d):
   for cut in cuts[j]:
    left=[i for i in indices if x[i][j]<=cut];nl=len(left);nr=len(indices)-nl
    if min(nl,nr)<15:continue
    sl=sum(residual[i] for i in left);gain=sl*sl/nl+(total-sl)**2/nr-total*total/len(indices)
    if best is None or gain>best[0]:best=(gain,j,cut,left)
  if best is None or best[0]<=0:return {'value':.1*mean}
  _,j,cut,left=best;right=[i for i in indices if x[i][j]>cut]
  return dict(feature=j,cut=cut,left=tree(left,residual,depth+1),right=tree(right,residual,depth+1))
 for epoch in range(60):
  residual=[a-b for a,b in zip(y,pred)];t=tree(list(range(len(y))),residual);trees.append(t)
  pred=[p+score_tree(t,row) for p,row in zip(pred,x)]
 cal_indices=[i for i,p in enumerate(cs) if p['prediction']['status']=='predicted']
 scores=[bias+sum(score_tree(t,cs[i]['features']) for t in trees) for i in cal_indices]
 selected=threshold(scores,[cy[i] for i in cal_indices]);margin=threshold([cs[i]['prediction']['margin'] for i in cal_indices],[cy[i] for i in cal_indices])
 model=dict(version=1,bias=bias,threshold=selected['threshold'] if selected else None,trees=trees)
 qa.write(RUN/'support-model.json',model)
 audit=dict(fit_rows=len(fs),fit_selected=len(take),fit_selected_right=sum(y),cal_rows=len(cs),cal_selected=len(cal_indices),cal_selected_right=sum(cy[i] for i in cal_indices),support=selected,margin=margin,features=d,warning='Threshold selection only; scores are not probabilities or entailment.')
 qa.write(RUN/'train-audit.json',audit); print(json.dumps(audit),flush=True)
 # All fit/cal diagnostics, including exactness labels, remain offline.
 qa.lines(RUN/'train-diagnostics.jsonl',[dict(split=n,id=r['id'],correct=v,**p) for n,rs,ps,ys in [('fit',fr,fs,fy),('cal',cr,cs,cy)] for r,p,v in zip(rs,ps,ys)])
def score_tree(t,x):return t['value'] if 'value' in t else score_tree(t['left'] if x[t['feature']]<=t['cut'] else t['right'],x)
def evaluate(size):
 rows=sorted(qa.rows(qa.load('dev-v1.1.json')['data']),key=qa.order)
 if size=='sample':rows=rows[:200]
 qa.lines(RUN/(size+'.input.jsonl'),[dict(context=r['context'],question=r['question']) for r in rows])
 ps=infer(size,support=True,features=False)
 baseline=read(BASE/'full.candidate.outputs.jsonl')[:len(rows)]
 audit=json.loads((RUN/'train-audit.json').read_text());mt=audit['margin']['threshold'] if audit['margin'] else None
 margin=[];transitions=[]
 for r,b,p in zip(rows,baseline,ps):
  assert all(b[k]==p[k] for k in ['proposal','evidence','score','margin'])
  if b['status']=='refused':assert p==b
  elif p['status']=='predicted':assert p==b
  else:
   assert p['answer']==''
   right=max(qa.SCORE['compute_exact'](a['text'],b['answer']) for a in r['answers'])
   transitions.append(dict(id=r['id'],transition='correct_to_refused' if right else 'wrong_to_refused',old_answer=b['answer']))
  m=dict(b)
  if m['status']=='predicted' and (mt is None or m['margin']<mt):m.update(status='refused',answer='',reason='margin_only')
  margin.append(m)
 results={name:qa.metrics(rows,p,size+'.'+name) for name,p in [('v1',baseline),('support',ps),('margin',margin)]}
 c=results['support'];m=results['margin']
 results['acceptance']=bool(audit['support'] and c['right']>=400 and (c['precision'] or 0)>=.8 and c['right']>m['right'] and (m['precision'] is None or c['precision']>=m['precision']-.02))
 results['transitions']=dict(collections.Counter(t['transition'] for t in transitions));results['old_refusals_preserved']=True
 qa.write(RUN/(size+'.json'),results);qa.write(RUN/(size+'.transitions.json'),transitions);print(json.dumps(results,indent=2),flush=True)
 if size=='full':
  # Independently replay old v1 through the unchanged library adapter.
  cmd=[str(BINARY.parent/'reading-qa'),'predict','--input',str(BASE/'full.input.jsonl'),'--model',str(BASE/'model.json')]
  out=RUN/'v1-replay.jsonl'
  with out.open('w') as f:subprocess.run(cmd,stdout=f,check=True,timeout=590)
  assert out.read_bytes()==(BASE/'full.candidate.outputs.jsonl').read_bytes()
  print('v1 byte equality: all 10570 rows',flush=True)
def probes():
 cases=[]
 def add(name,c,q,expected):cases.append(dict(name=name,context=c,question=q,expected=expected))
 for c,subject,obj in [('Nora thanked Felix.','Nora','Felix'),('Felix thanked Nora.','Felix','Nora')]:
  add('subject_'+subject,c,'Who thanked Felix?', 'Nora' if subject=='Nora' else None)
  add('object_'+obj,c,'Who did Nora thank?', 'Felix' if subject=='Nora' else None)
 add('unrelated_predicate','Nora painted the bridge in 2008.','When did Nora demolish the bridge?',None)
 add('changed_relation_positive','Nora demolished the bridge in 2008.','When did Nora demolish the bridge?','2008')
 add('negation','Nora never demolished the bridge in 2008.','When did Nora demolish the bridge?',None)
 add('person_kind','Tessa visited Bergen in 2017.','Who visited Bergen?','Tessa')
 add('place_kind','Tessa visited Bergen in 2017.','Where did Tessa visit?','Bergen')
 add('date_kind','Tessa visited Bergen in 2017.','When did Tessa visit Bergen?','2017')
 add('person_vs_number','2017 was printed on the wall.','Who was printed on the wall?',None)
 add('place_vs_number','Tessa counted 2017 stones.','Where did Tessa count stones?',None)
 add('role_swap_positive','Ivo hired Mara in 2003.','Who hired Mara?','Ivo')
 add('role_swap_negative','Mara hired Ivo in 2003.','Who hired Mara?',None)
 add('unicode','Maël greeted Zoë.','Who greeted Zoë?','Maël')
 qa.lines(RUN/'probes.input.jsonl',[dict(context=c['context'],question=c['question']) for c in cases])
 b=infer('probes',features=False);p=infer('probes',support=True,features=False)
 for c,old,new in zip(cases,b,p):
  c.update(baseline=old,candidate=new,passes=(new['status']=='refused' if c['expected'] is None else new['answer']==c['expected']))
 qa.write(RUN/'probes.json',cases);print('probes',sum(c['passes'] for c in cases),'/',len(cases),flush=True)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('stage',choices=['train','sample','full','probes']);a=p.parse_args()
 if a.stage=='train':train()
 elif a.stage=='probes':probes()
 else:evaluate(a.stage)
