#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Ownership-local orchestration; unchanged official evaluator imported from qa/run.py."""
import os,pathlib,importlib.util,json,hashlib,subprocess,sys,collections
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[2]
RUN=pathlib.Path(os.environ.get('QA_SYNTAX_RUN','/home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run/experiment'))
BASE=RUN.parent/'baseline'
REPORT=HERE/'reports'
os.environ.setdefault('SQUAD_SOURCE','/home/gsc/.cache/glassmind/continuous-20261010/readingqa-source')
os.environ['QA_ARTIFACTS_DIR']=str(RUN)
spec=importlib.util.spec_from_file_location('original_qa',HERE.parent/'qa/run.py');q=importlib.util.module_from_spec(spec);spec.loader.exec_module(q)
BIN=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release'
def write(p,x):q.write(p,x)
def infer(name,inp,old=False):
    out=RUN/(name+'.outputs.jsonl')
    cmd=[str(BIN/('reading-qa' if old else 'reading-qa-syntax'))]
    if not old:cmd+=['--syntax',str(BASE/'syntax.bin')]
    cmd+=['predict','--input',str(inp),'--model',str(BASE/'model.json' if old else RUN/'model.json')]
    with out.open('w') as f:subprocess.run(cmd,stdout=f,check=True,timeout=590)
    return [json.loads(l) for l in out.read_text().splitlines()]
def freeze():
    files=[RUN/'model.json',BIN/'reading-qa-syntax',BIN/'reading-qa',BASE/'syntax.bin',q.SOURCE/'evaluate-v2.0.py',HERE/'PREREGISTER.md',HERE/'probes.json',HERE/'run.py',ROOT/'crates/reading/src/qa.rs',ROOT/'crates/reading/src/qa/syntax_v3.rs',ROOT/'crates/reading/src/bin/reading-qa-syntax.rs']
    files += [q.SOURCE/n for n in ['train-v1.1.json','dev-v1.1.json','provenance.json','LICENSE','revision.txt']]
    files += [RUN/n for n in ['fit.train.jsonl','fit.input.jsonl','cal.input.jsonl','cal.labels.json','selection.json']]
    files += [BASE/n for n in ['manifest.json','model.json','full.input.jsonl','full.candidate.outputs.jsonl']]
    files += [ROOT/n for n in ['Cargo.lock','crates/reading/Cargo.toml','tools/reading/qa/run.py']]
    write(REPORT/'freeze.json',[dict(path=str(p),bytes=p.stat().st_size,sha256=q.digest(p.read_bytes())) for p in files])
def verify_freeze():
    for x in json.loads((REPORT/'freeze.json').read_text()):assert q.digest(pathlib.Path(x['path']).read_bytes())==x['sha256'],x['path']
def calibrate():
    rs=json.loads((RUN/'cal.labels.json').read_text());ps=infer('cal',RUN/'cal.input.jsonl');assert len(rs)==len(ps)==1000
    ranked=[]
    for r,p in zip(rs,ps):
        raw=q.proposal(r,p);correct=max(q.SCORE['compute_exact'](a['text'],raw) for a in r['answers'])
        if p['proposal'] and p['reason']!='no_lexical_support':ranked.append((p['margin'],correct))
    ranked.sort(reverse=True);right=0;front=[];selected=None
    for i,(margin,c) in enumerate(ranked):
        right+=c
        if i+1<len(ranked) and ranked[i+1][0]==margin:continue
        row=dict(threshold=margin,accepted=i+1,right=right,wrong=i+1-right,precision=right/(i+1));front.append(row)
        if i+1>=30 and right/(i+1)>=.5:selected=row
    write(RUN/'cal.frontier.json',front);write(REPORT/'calibration.json',{'selected':selected,'total':1000})
    if selected:
        m=json.loads((RUN/'model.json').read_text());m['ranker']['threshold']=selected['threshold'];write(RUN/'model.json',m)
    freeze();print(json.dumps(selected));return selected
def state(r,p,raw=False):
    text=q.proposal(r,p) if raw else p['answer']
    if not raw and p['status']!='predicted':return 'refused'
    gold=[a['text'] for a in r['answers'] if q.SCORE['normalize_answer'](a['text'])]
    return 'right' if text and any(q.SCORE['compute_exact'](a,text) for a in gold) else 'wrong'
def evaluate(size):
    verify_freeze();assert json.loads((REPORT/'calibration.json').read_text())['selected'] is not None
    rs=sorted(q.rows(q.load('dev-v1.1.json')['data']),key=q.order);assert len(rs)==10570
    old=[json.loads(l) for l in (BASE/'full.candidate.outputs.jsonl').read_text().splitlines()];assert len(old)==10570
    if size=='sample':rs=rs[:200];old=old[:200]
    inp=RUN/(size+'.input.jsonl');q.lines(inp,[dict(context=r['context'],question=r['question']) for r in rs])
    if size=='full':assert inp.read_bytes()==(BASE/'full.input.jsonl').read_bytes()
    ps=infer(size,inp);assert len(ps)==len(rs)
    result={'baseline':q.metrics(rs,old,size+'.v1'),'candidate':q.metrics(rs,ps,size+'.v3')}
    transitions=[]
    for r,b,c in zip(rs,old,ps):transitions.append(dict(id=r['id'],old_raw=q.proposal(r,b),new_raw=q.proposal(r,c),old_selected=b['answer'],new_selected=c['answer'],raw=state(r,b,True)+'->'+state(r,c,True),selected=state(r,b)+'->'+state(r,c)))
    write(RUN/(size+'.transitions.json'),transitions)
    result['transitions']={k:dict(collections.Counter(t[k] for t in transitions)) for k in ['raw','selected']}
    c=result['candidate'];result['aggregate_gate']=c['raw']['exact']*len(rs)/100>=2744 and c['raw']['f1']>=36.5184250480 and c['right']>=815 and c['wrong']<=791 if size=='full' else None
    write(REPORT/(size+'.json'),result);print(json.dumps(result,indent=2))
    if size=='full':
        infer('v1-replay',inp,True);assert (RUN/'v1-replay.outputs.jsonl').read_bytes()==(BASE/'full.candidate.outputs.jsonl').read_bytes();print('v1 full replay byte-identical')
def probes():
    verify_freeze();rs=json.loads((HERE/'probes.json').read_text());inp=RUN/'probes.input.jsonl';q.lines(inp,[dict(context=r['context'],question=r['question']) for r in rs]);b=infer('probes.v1',inp,True);c=infer('probes.v3',inp)
    for r,old,new in zip(rs,b,c):
        r.update(v1=old,v3=new,raw_v1=q.proposal(r,old),raw_v3=q.proposal(r,new));r['raw_correct']=r['expected'] is not None and bool(q.SCORE['compute_exact'](r['expected'],r['raw_v3']));r['selected_correct']=new['status']=='refused' if r['expected'] is None else bool(q.SCORE['compute_exact'](r['expected'],new['answer']))
    report={'positive_raw_correct':sum(r['raw_correct'] for r in rs if r['positive_role']),'positive_total':8,'newly_selected_mismatches':[r['name'] for r in rs if r['clear_mismatch'] and r['v1']['status']=='refused' and r['v3']['status']=='predicted'],'cases':rs};report['gate']=report['positive_raw_correct']>=6 and not report['newly_selected_mismatches'];write(REPORT/'probes.json',report);print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
if __name__=='__main__':
    stage=sys.argv[1]
    if stage=='calibrate':calibrate()
    elif stage=='probes':probes()
    else:evaluate(stage)
