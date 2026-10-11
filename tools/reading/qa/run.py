#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Offline selection/calibration/evaluation; inference receives context/question only."""
import argparse, ast, collections, hashlib, json, os, pathlib, re, string, subprocess
HERE = pathlib.Path(__file__).resolve().parent
SOURCE = pathlib.Path(os.environ.get('SQUAD_SOURCE', HERE.parents[3] / 'readingqa-source'))
ART = pathlib.Path(os.environ.get('QA_ARTIFACTS_DIR', HERE / 'artifacts'))
REPORT = pathlib.Path(os.environ.get('QA_REPORT_DIR', HERE / 'reports'))

def digest(b): return hashlib.sha256(b).hexdigest()
def write(path, value): path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
def lines(path, rows): path.write_text(''.join(json.dumps(r, ensure_ascii=False)+'\n' for r in rows))
def load(name): return json.loads((SOURCE/name).read_text())
def official():
    source = (SOURCE/'evaluate-v2.0.py').read_bytes()
    assert digest(source) == '710840ce2c2c61716334c056777032f24660b83eb9869a451036e6a24d8103c4'
    # Execute the exact, unmodified official function ASTs. Avoid optional NumPy/plot imports.
    names = {'normalize_answer','get_tokens','compute_exact','compute_f1','get_raw_scores','make_eval_dict'}
    nodes = [n for n in ast.parse(source).body if isinstance(n, ast.FunctionDef) and n.name in names]
    assert {n.name for n in nodes} == names
    ns = dict(collections=collections,re=re,string=string)
    exec(compile(ast.Module(body=nodes,type_ignores=[]),str(SOURCE/'evaluate-v2.0.py'),'exec'), ns)
    return ns
SCORE = official()
def rows(articles):
    return [dict(id=q['id'],context=p['context'],question=q['question'],answers=q['answers'])
            for a in articles for p in a['paragraphs'] for q in p['qas']]
def article_hash(a): return digest('\n'.join(p['context'] for p in a['paragraphs']).encode())
def order(row): return digest(row['id'].encode())
def prepare():
    ART.mkdir(parents=True, exist_ok=True); REPORT.mkdir(parents=True, exist_ok=True)
    train = load('train-v1.1.json')['data']
    assert len(train)==442 and len(rows(train))==87599
    articles=sorted(train,key=article_hash)
    selections={}
    for name,selected,cap in [('fit',articles[:64],4000),('cal',articles[64:80],1000)]:
        selected_rows=sorted(rows(selected),key=order)[:cap]
        selections[name]=dict(article_hashes=[article_hash(a) for a in selected],ids=[r['id'] for r in selected_rows],available_questions=len(rows(selected)),selected_questions=len(selected_rows))
        lines(ART/(name+'.input.jsonl'),[dict(context=r['context'],question=r['question']) for r in selected_rows])
        if name=='fit':
            fit=[]
            for r in selected_rows:
                a=r['answers'][0]; start=len(r['context'][:a['answer_start']].encode()); end=start+len(a['text'].encode())
                assert r['context'].encode()[start:end].decode()==a['text']
                fit.append(dict(context=r['context'],question=r['question'],answer=dict(start=start,end=end)))
            lines(ART/'fit.train.jsonl',fit)
        else: write(ART/'cal.labels.json',selected_rows)
    assert not set(selections['fit']['article_hashes']) & set(selections['cal']['article_hashes'])
    selections['train_source_sha256']=digest((SOURCE/'train-v1.1.json').read_bytes())
    write(REPORT/'selection.json',selections)
    print(json.dumps({n:{k:v for k,v in s.items() if k not in ('ids','article_hashes')} for n,s in selections.items() if isinstance(s,dict)}))

def infer(binary, inputs, model, output):
    cmd=[str(binary),'predict','--input',str(inputs)]
    if model: cmd+=['--model',str(model)]
    with output.open('w') as f: subprocess.run(cmd,stdout=f,check=True,timeout=590)
    return [json.loads(l) for l in output.read_text().splitlines()]

def proposal(r,p):
    span=p['proposal']
    if span is None: return ''
    answer=r['context'].encode()[span['start']:span['end']].decode()
    evidence=p['evidence']; assert evidence['start']<=span['start']<=span['end']<=evidence['end']
    if p['status']=='predicted': assert p['answer']==answer
    else: assert p['answer']==''
    return answer

def calibrate(binary):
    rs=json.loads((ART/'cal.labels.json').read_text())
    predictions=infer(binary,ART/'cal.input.jsonl',ART/'model.json',ART/'cal.outputs.jsonl')
    assert len(rs)==len(predictions)==1000
    ranked=[]
    for r,p in zip(rs,predictions):
        text=proposal(r,p)
        correct=max(SCORE['compute_exact'](a['text'],text) for a in r['answers'])
        if p['proposal'] and p['reason']!='no_lexical_support': ranked.append((p['margin'],correct))
    ranked.sort(reverse=True)
    selected=None; right=0
    for i,(margin,correct) in enumerate(ranked):
        right+=correct
        if i+1<len(ranked) and ranked[i+1][0]==margin: continue
        if i+1>=30 and right/(i+1)>=0.5: selected=dict(threshold=margin,accepted=i+1,right=right,wrong=i+1-right,precision=right/(i+1),coverage=(i+1)/len(rs))
    model=json.loads((ART/'model.json').read_text())
    if selected: model['threshold']=selected['threshold']
    else: selected=dict(threshold=model['threshold'],accepted=0,right=0,wrong=0,precision=None,coverage=0)
    write(ART/'model.json',model)
    selected.update(total=len(rs),fit_rows=model['fit_rows'],representable_rows=model['representable_rows'],rule='maximum coverage at >=0.50 normalized exact precision, >=30 accepted; tied margins included together',warning='TRAIN calibration selection estimate; not a probability or general unanswerable calibration')
    write(REPORT/'calibration.json',selected); print(json.dumps(selected))

def metrics(rs,ps,prefix):
    assert len(rs)==len(ps)
    ids=[r['id'] for r in rs]; assert len(set(ids))==len(ids)
    raw={r['id']:proposal(r,p) for r,p in zip(rs,ps)}
    accepted={r['id']:p['answer'] for r,p in zip(rs,ps)}
    # Exact ID-set check BEFORE scorer; missing IDs must never reduce denominator.
    assert set(raw)==set(accepted)==set(ids)
    write(ART/(prefix+'.raw.json'),raw); write(ART/(prefix+'.accepted.json'),accepted)
    dataset=[dict(paragraphs=[dict(qas=[dict(id=r['id'],answers=r['answers']) for r in rs])])]
    er,fr=SCORE['get_raw_scores'](dataset,raw); ea,fa=SCORE['get_raw_scores'](dataset,accepted)
    assert len(er)==len(fr)==len(ea)==len(fa)==len(rs)
    n=sum(p['status']=='predicted' for p in ps); right=sum(ea.values())
    return dict(total=len(rs),raw=SCORE['make_eval_dict'](er,fr),accepted=SCORE['make_eval_dict'](ea,fa),right=right,refused=len(rs)-n,wrong=n-right,precision=right/n if n else None,coverage=n/len(rs))

def evaluate(binary,size):
    dev=load('dev-v1.1.json')['data']; rs=sorted(rows(dev),key=order)
    assert len(dev)==48 and len(rs)==10570
    if size=='sample': rs=rs[:200]
    inputs=ART/(size+'.input.jsonl'); lines(inputs,[dict(context=r['context'],question=r['question']) for r in rs])
    results={}
    for name,model in [('baseline',None),('candidate',ART/'model.json')]:
        ps=infer(binary,inputs,model,ART/(size+'.'+name+'.outputs.jsonl'))
        results[name]=metrics(rs,ps,size+'.'+name)
    b,c=results['baseline'],results['candidate']
    results['gains_pp']={k:c['raw'][k]-b['raw'][k] for k in ('exact','f1')}
    results['passes_frozen_gate']=results['gains_pp']['exact']>=2 and results['gains_pp']['f1']>=3 and c['coverage']>=.01 and (c['precision'] or 0)>=.4
    results['selection']='SHA256(questionID) ascending; first200 for sample, all10570 for full'
    write(REPORT/(size+'.json'),results); print(json.dumps(results,indent=2))

def independent(binary):
    cases=[
      dict(name='unicode',context='Élodie visited Zürich in 2018.',question='Who visited Zürich?',expected='Élodie'),
      dict(name='no_support',context='Lena owns a bicycle.',question='When did the volcano erupt?',expected=None),
      dict(name='ambiguous',context='Ana visited Lima. Bea visited Lima.',question='Who visited Lima?',expected=None),
      dict(name='multioccurrence',context='Ana visited Lima. Ana visited Lima.',question='Who visited Lima?',expected='Ana'),
      dict(name='trailing_text',context='Mira founded the club in 2004. The unrelated bakery sells bread.',question='When did Mira found the club?',expected='2004'),
      dict(name='unsupported_relation',context='Lena painted a bicycle.',question='When did Lena buy the bicycle?',expected=None),
    ]
    lines(ART/'independent.input.jsonl',[dict(context=c['context'],question=c['question']) for c in cases])
    ps=infer(binary,ART/'independent.input.jsonl',ART/'model.json',ART/'independent.outputs.jsonl')
    for c,p in zip(cases,ps): c.update(prediction=p,raw=proposal(c,p),correct_behavior=(p['status']=='refused' if c['expected'] is None else p['answer']==c['expected']))
    write(REPORT/'independent.json',cases); print(json.dumps(cases,indent=2,ensure_ascii=False))

if __name__=='__main__':
    parser=argparse.ArgumentParser(); parser.add_argument('stage',choices=['prepare','calibrate','sample','full','independent']); parser.add_argument('--binary',type=pathlib.Path)
    args=parser.parse_args()
    if args.stage=='prepare': prepare()
    elif args.stage=='calibrate': calibrate(args.binary)
    elif args.stage=='independent': independent(args.binary)
    else: evaluate(args.binary,args.stage)
