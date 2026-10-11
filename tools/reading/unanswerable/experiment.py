#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Offline experiment only. Runtime receives strict context/question JSONL."""
import collections, hashlib, importlib.util, json, math, os, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]; WORK=ROOT.parent
RUN=Path(os.environ.get('NULL_RUN',WORK/'unanswerable-run/experiment'))
SRC=WORK/'readingqa2-source'; BASE=WORK/'unanswerable-run/baseline'; REPORT=ROOT/'tools/reading/unanswerable/reports'
os.environ['SQUAD_SOURCE']=str(SRC)
spec=importlib.util.spec_from_file_location('qa_run',ROOT/'tools/reading/qa/run.py'); qa=importlib.util.module_from_spec(spec);spec.loader.exec_module(qa)
BIN=Path(os.environ['CARGO_TARGET_DIR'])/'release/reading-qa'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def dump(p,x):p.write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n')
def rows(articles):
 return [dict(id=q['id'],title=a['title'],context=p['context'],question=q['question'],answers=q['answers'],impossible=q['is_impossible']) for a in articles for p in a['paragraphs'] for q in p['qas']]
def lines(p,x):p.write_text(''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in x))
def get(p):return json.loads(p.read_text())
def infer(name,rs,extra):
 inp=RUN/(name+'.input.jsonl');out=RUN/(name+'.output.jsonl')
 assert not out.exists(), 'Never overwrite existing predictions'
 lines(inp,[dict(context=r['context'],question=r['question']) for r in rs])
 with out.open('w') as f:subprocess.run([str(BIN),'predict','--input',str(inp),'--model',str(BASE/'model.json')]+extra,stdout=f,check=True,timeout=570)
 ps=[json.loads(s) for s in out.read_text().splitlines()];assert len(ps)==len(rs);return ps

def prepare():
 assert sha(BASE/'model.json')=='b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5'
 for x in get(ROOT/'tools/reading/qa2/source.json')['files']:assert sha(SRC/x['file'])==x['sha256']
 old=get(ROOT/'tools/reading/qa/reports/selection.json');ex=set(old['fit']['article_hashes']+old['cal']['article_hashes'])
 titles={a['title'] for a in get(WORK/'readingqa-source/train-v1.1.json')['data'] if qa.article_hash(a) in ex};assert len(titles)==80
 aa=sorted([a for a in get(SRC/'train-v2.0.json')['data'] if a['title'] not in titles],key=lambda a:qa.digest(a['title'].encode()))
 dev=rows(get(SRC/'dev-v2.0.json')['data']) # overlap metadata only; no outcomes inspected
 groups={'fit':rows(aa[:96]),'cal':rows(aa[96:128]),'dev':dev}
 sets={k:{r['context'] for r in rs} for k,rs in groups.items()}
 shared=(sets['fit']&sets['cal'])|(sets['fit']&sets['dev'])|(sets['cal']&sets['dev'])
 report=dict(excluded_titles=sorted(titles),shared_contexts=len(shared),removed={},selection={})
 for name,cap,arts in [('fit',8000,aa[:96]),('cal',2000,aa[96:128])]:
  clean=[r for r in groups[name] if r['context'] not in shared];report['removed'][name]=len(groups[name])-len(clean)
  rs=sorted(clean,key=qa.order)[:cap];groups[name]=rs;dump(RUN/(name+'.labels.json'),rs)
  report['selection'][name]=dict(titles=[a['title'] for a in arts],ids=[r['id'] for r in rs],rows=len(rs),impossible=sum(r['impossible'] for r in rs),available=len(clean))
 for field in ('title','context','id'):
  for a,b in [('fit','cal'),('fit','dev'),('cal','dev')]:assert not {r[field] for r in groups[a]}&{r[field] for r in groups[b]},(field,a,b)
 report['overlap_assertions']='title/context/id pairwise zero after TRAIN-side removal'
 dump(REPORT/'selection.json',report);print({k:{x:v for x,v in s.items() if x not in ('titles','ids')} for k,s in report['selection'].items()},flush=True)
 for name in ('fit','cal'):
  ps=infer(name,groups[name],['--null-features']);print(name,'features',len(ps),flush=True)

def labelled(name):
 rs=get(RUN/(name+'.labels.json'));ps=[json.loads(s) for s in (RUN/(name+'.output.jsonl')).read_text().splitlines()]
 assert len(rs)==len(ps)
 result=[]
 for r,p in zip(rs,ps,strict=True):
  raw=qa.proposal(r,p['prediction']);gold=[a['text'] for a in r['answers'] if qa.SCORE['normalize_answer'](a['text'])]
  correct=not r['impossible'] and bool(qa.SCORE['normalize_answer'](raw)) and any(qa.SCORE['compute_exact'](a,raw) for a in gold)
  result.append((r,p,int(correct)))
 return result

def train():
 assert not (RUN/'first-weights.json').exists(), 'One training run only'
 data=labelled('fit');w=[0.0]*8192;acc=[1.0]*8192
 for epoch in range(12):
  loss=0.0
  for r,p,y in data:
   fs=p['features']
   z=sum(w[i]*x for i,x in fs);prob=1/(1+math.exp(-max(-40,min(40,z))))
   loss+=-math.log(max(1e-15,prob if y else 1-prob))
   for i,x in fs:
    g=(prob-y)*x+.0001*w[i];acc[i]+=g*g;w[i]-=.08*g/math.sqrt(acc[i])
  print('epoch',epoch+1,'loss',loss/len(data),flush=True)
 dump(RUN/'first-weights.json',dict(version=1,weights=w,threshold=0.0))
 dump(REPORT/'training.json',dict(rows=len(data),positives=sum(y for r,p,y in data),genuine_unanswerables=sum(r['impossible'] for r,p,y in data),wrong_span_negatives=sum(not r['impossible'] and not y for r,p,y in data),epochs=12,first_weights_sha256=sha(RUN/'first-weights.json')))

def calibrate():
 m=get(RUN/'first-weights.json');data=labelled('cal');totalnull=sum(r['impossible'] for r,p,y in data)
 ranked=sorted([(sum(m['weights'][i]*x for i,x in p['features']),y,int(r['impossible']),r['id']) for r,p,y in data if p['features']],reverse=True)
 right=wrongnull=0;front=[];chosen=None
 for i,(s,y,n,_) in enumerate(ranked):
  right+=y;wrongnull+=n
  if i+1<len(ranked) and ranked[i+1][0]==s:continue
  row=dict(threshold=s,accepted=i+1,correct=right,wrong=i+1-right,precision=right/(i+1),strict_correct=right+totalnull-wrongnull,expected_refusals=totalnull-wrongnull)
  front.append(row)
  if row['precision']>=.6 and right>=100 and row['strict_correct']>totalnull:chosen=row
 dump(RUN/'cal-frontier.json',front)
 report=dict(gate_pass=chosen is not None,selected=chosen,total=len(data),always_refuse=totalnull,baseline=metrics([r for r,p,y in data],[p['prediction'] for r,p,y in data]),best_precision_at_100_correct=max((x for x in front if x['correct']>=100),key=lambda x:x['precision'],default=None))
 dump(REPORT/'calibration.json',report);print(json.dumps(report),flush=True)
 if chosen:
  m['threshold']=chosen['threshold'];dump(RUN/'model.json',m)
  dump(REPORT/'freeze.json',dict(files={str(p):sha(p) for p in [RUN/'model.json',BIN,ROOT/'crates/reading/src/qa.rs',ROOT/'crates/reading/src/qa/null.rs',ROOT/'crates/reading/src/bin/reading-qa.rs',Path(__file__),ROOT/'tools/reading/unanswerable/PREREGISTER.md',ROOT/'tools/reading/unanswerable/probes.json']}))
 else:print('REJECTED CAL; candidate DEV prohibited',flush=True)

def metrics(rs,ps):
 assert len(rs)==len(ps)==len({r['id'] for r in rs})
 preds={r['id']:p['answer'] for r,p in zip(rs,ps,strict=True)}
 dataset=[dict(paragraphs=[dict(qas=[dict(id=r['id'],answers=r['answers']) for r in rs])])]
 ex,f1=qa.SCORE['get_raw_scores'](dataset,preds);assert set(ex)==set(f1)==set(preds)
 counts=collections.Counter();statuses=[];coinc=[]
 for r,p in zip(rs,ps,strict=True):
  qa.proposal(r,p)
  status=('expected_refusal' if r['impossible'] else 'unsupported') if p['status']=='refused' else ('correct' if not r['impossible'] and ex[r['id']] else 'wrong')
  counts[status]+=1;statuses.append(status)
  if bool(ex[r['id']])!=(status in ('expected_refusal','correct')):coinc.append(dict(id=r['id'],answer=p['answer']))
 result=dict(total=len(rs),counts=dict(counts),precision=counts['correct']/max(1,counts['correct']+counts['wrong']),strict_correct=counts['correct']+counts['expected_refusal'],official=qa.SCORE['make_eval_dict'](ex,f1),normalization_coincidences=coinc)
 return result

def probes():
 rs=get(ROOT/'tools/reading/unanswerable/probes.json');ps=infer('probes',rs,['--null-features']);m=get(RUN/('model.json' if (RUN/'model.json').exists() else 'first-weights.json'))
 for r,p in zip(rs,ps,strict=True):
  raw=qa.proposal(r,p['prediction']);s=sum(m['weights'][i]*x for i,x in p['features']);selected=(raw if p['features'] and s>=m['threshold'] else '') if (RUN/'model.json').exists() else None
  r.update(raw=raw,baseline=p['prediction'],score=s,selected=selected,selected_behavior_pass=(selected==r['expected'] if r['expected'] is not None else selected=='') if selected is not None else None)
 dump(REPORT/'probes.json',dict(note='No selected behavior asserted without passing CAL threshold. Raw spans and scores are diagnostic, not entailment.',cases=rs))

def dev(size):
 assert get(REPORT/'calibration.json')['gate_pass']
 for p,h in get(REPORT/'freeze.json')['files'].items():assert sha(p)==h
 rs=sorted(rows(get(SRC/'dev-v2.0.json')['data']),key=qa.order);assert len(rs)==11873
 old=[json.loads(s) for s in (BASE/'full.outputs.jsonl').read_text().splitlines()];assert len(old)==len(rs)
 assert sha(BASE/'full.outputs.jsonl')=='6dd02aa2757011bec9971e47aef04e08260426f089eea004396ac13f0f9666a4'
 if size=='sample':rs=rs[:200];old=old[:200]
 ps=infer(size,rs,['--null-model',str(RUN/'model.json')]);c=metrics(rs,ps);b=metrics(rs,old)
 for r,p,o in zip(rs,ps,old,strict=True):
  for key in ('proposal','evidence','score','margin'):assert p[key]==o[key]
 dump(RUN/(size+'.transitions.json'),[dict(id=r['id'],old=o,new=p) for r,o,p in zip(rs,old,ps,strict=True)])
 c['gate_pass']=c['counts'].get('correct',0)>=433 and c['counts'].get('wrong',0)<1236 and c['precision']>=.5 and c['strict_correct']>5945 and c['official']['exact']>=50.5
 dump(REPORT/(size+'.json'),dict(baseline=b,candidate=c));print(json.dumps(c),flush=True)

if __name__=='__main__':
 RUN.mkdir(exist_ok=True,parents=True);REPORT.mkdir(exist_ok=True,parents=True)
 {'prepare':prepare,'train':train,'calibrate':calibrate,'probes':probes,'sample':lambda:dev('sample'),'full':lambda:dev('full')}[sys.argv[1]]()
