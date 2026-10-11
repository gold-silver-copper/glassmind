#!/usr/bin/env python3
"""One frozen fit and separate saved-prediction scoring. Offline only."""
import collections, hashlib, json, math, os, re, shutil, subprocess, sys
from pathlib import Path
import pyarrow.parquet as pq
ROOT=Path(__file__).resolve().parents[3]
RUN=Path(os.environ['PROPOSAL_RUN'])
SOURCE=Path(os.environ['PROPOSAL_SOURCE'])
TARGET=Path(os.environ['CARGO_TARGET_DIR'])
sys.path.insert(0,str(ROOT/'tools/quantities/typed-label-metric'))
from metric import parse_gold, compare
CONST=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
def dump(path,value):
    path.write_text(json.dumps(value,sort_keys=True,separators=(',',':'))+'\n')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def manifest(name,paths):dump(RUN/name,{str(p):{'sha256':sha(p),'bytes':p.stat().st_size} for p in paths})
def jsonl(path,rows):path.write_text(''.join(json.dumps(x,separators=(',',':'))+'\n' for x in rows))
def run(exe,args,rows,path):
    data=''.join(json.dumps(x,separators=(',',':'))+'\n' for x in rows)
    p=subprocess.run([str(exe),*args],input=data,text=True,capture_output=True,check=True,timeout=500)
    path.write_text(p.stdout)
    out=[json.loads(x) for x in p.stdout.splitlines()]
    assert len(out)==len(rows)
    return out

def program(formula):
    steps=[]
    for i,s in enumerate(formula.rstrip('|').split('|')):
        m=re.fullmatch(r'(add|subtract|multiply|divide)\((n\d+|#\d+|const_\d+(?:_\d+)?),(n\d+|#\d+|const_\d+(?:_\d+)?)\)',s)
        if not m:return None
        def arg(a):
            if a.startswith('n'):
                n=int(a[1:]);assert n<16;return {'kind':'input','index':n}
            if a.startswith('#'):
                n=int(a[1:]);assert n<i;return {'kind':'step','index':n}
            return {'kind':'constant','index':CONST.index(a[6:].replace('_','.'))}
        try:steps.append({'op':m[1],'left':arg(m[2]),'right':arg(m[3])})
        except (AssertionError,ValueError):return None
    return {'steps':steps} if 1<=len(steps)<=6 else None

def key(p):return json.dumps(p,sort_keys=True,separators=(',',':'))
def group(q):return hashlib.sha256(re.sub('[0-9]+','#',' '.join(q.lower().split())).encode()).hexdigest()
def words(q):return re.findall('[a-z]+',q.translate(str.maketrans('ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')))
def features(q,n):
    w=words(q); fs=['u:'+x for x in w]+['b:'+a+' '+b for a,b in zip(w,w[1:])]+['n:'+str(n)]
    def h(s):
        v=14695981039346656037
        for b in s.encode():v=((v^b)*1099511628211)&((1<<64)-1)
        return v%8192
    return sorted(set(map(h,fs)))
def grade(row,out):
    _,gold=parse_gold(row['options'],row['correct'])
    # Explicit proposal metric adapter; execution does NOT establish an answer.
    actual={'status':'answer','value':out['value'],'unit':None} if out['status']=='proposed' else {'status':'refusal'}
    return compare(gold,actual)['grade']
def report(labels,outs,threshold=None):
    counts=collections.Counter(grade(r,o) for r,o in zip(labels,outs))
    selected=[(r,o) for r,o in zip(labels,outs) if threshold is not None and o['status']=='proposed' and o['score']>=threshold]
    correct=sum(grade(r,o)=='correct' for r,o in selected)
    return {'total':len(labels),'raw':{k:counts[k] for k in ['correct','refused','wrong','unscorable']},'raw_conservative_wrong':len(labels)-counts['correct']-counts['refused'], 'selected':len(selected),'selected_correct':correct,'selected_wrong':len(selected)-correct,'selected_precision':correct/len(selected) if selected else None,'threshold':threshold}
def threshold(labels,outs):
    groups=collections.defaultdict(list)
    for r,o in zip(labels,outs):
        if o['status']=='proposed':groups[o['score']].append(grade(r,o)=='correct')
    n=correct=0; chosen=None;front=[]
    for s,ys in sorted(groups.items(),reverse=True):
        n+=len(ys);correct+=sum(ys)
        if n>=30 and correct/n>=.9:chosen=s
        if n>=30:front.append((correct/n,n,correct,s))
    return chosen, max(front,default=None)

def main():
    RUN.mkdir(parents=True,exist_ok=True)
    assert not (RUN/'model.json').exists(), 'one fit only; do not overwrite prior candidate'
    # Source/binary snapshot before fit, no dev access here.
    frozen=RUN/'source';frozen.mkdir()
    files=list((ROOT/'crates/quantities/src').glob('*.rs'))+list((ROOT/'crates/quantities/examples').glob('equation*.rs'))+list(Path(__file__).parent.glob('*'))+[ROOT/'crates/quantities/Cargo.toml']
    for p in files:
        if p.is_file():
            dest=frozen/p.relative_to(ROOT);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest)
    binary=RUN/'glassmind-quantities';executor=RUN/'equation_executor'
    shutil.copy2(TARGET/'release/glassmind-quantities',binary);shutil.copy2(TARGET/'release/examples/equation_executor',executor)
    manifest('prefit-manifest.json',[*filter(Path.is_file,frozen.rglob('*')),binary,executor,SOURCE/'train.parquet',SOURCE/'validation.parquet',SOURCE/'source-card.md'])
    rows=pq.read_table(SOURCE/'train.parquet',columns=['Problem','linear_formula','options','correct']).to_pylist()
    groups=[group(r['Problem']) for r in rows];cal=[i for i,g in enumerate(groups) if int(g,16)%5==0];fit=[i for i,g in enumerate(groups) if int(g,16)%5!=0]
    dump(RUN/'split.json',{'fit':fit,'calibration':cal,'group_hashes':groups})
    ps=[program(r['linear_formula']) for r in rows]
    executions=run(executor,[],[{'question':r['Problem'],'program':p} for r,p in zip(rows,ps)],RUN/'train-executions.jsonl')
    cleaned=[];exclusions=collections.Counter()
    for i in fit:
        if ps[i] is None:exclusions['unsupported_annotation']+=1
        elif executions[i]['status']!='proposed':exclusions['binding_or_execution_refusal']+=1
        elif grade(rows[i],executions[i])!='correct':exclusions['label_'+grade(rows[i],executions[i])]+=1
        else:cleaned.append(i)
    counts=collections.Counter(key(ps[i]) for i in cleaned)
    bank=[p for p,n in sorted(counts.items(),key=lambda x:(-x[1],x[0])) if n>=3][:512]
    bankset=set(bank);eligible=[i for i in cleaned if key(ps[i]) in bankset]
    # Single TF-IDF centroid fit, all eligible FIT examples.
    fs={i:features(rows[i]['Problem'],len(executions[i]['numbers'])) for i in eligible}
    df=collections.Counter(f for v in fs.values() for f in v)
    idf=[math.log((1+len(eligible))/(1+df[f]))+1 for f in range(8192)]
    weights={p:collections.defaultdict(float) for p in bank}
    for i in eligible:
        norm=math.sqrt(sum(idf[f]**2 for f in fs[i]));w=weights[key(ps[i])]
        for f in fs[i]:w[f]+=idf[f]/norm
    classes=[]
    for p in bank:
        norm=math.sqrt(sum(v*v for v in weights[p].values()))
        classes.append({'program':json.loads(p),'weights':{str(f):v/norm for f,v in sorted(weights[p].items())}})
    model={'schema':'equation-centroid-v1','provenance':'TRAIN '+sha(SOURCE/'train.parquet')+'; plan '+sha(Path(__file__).parent/'PLAN.md')+'; grouped-fit-only; finite-centroid-v1','idf':idf,'classes':classes}
    dump(RUN/'model.json',model)
    assert (RUN/'model.json').stat().st_size<=32*1024*1024
    dump(RUN/'fit-report.json',{'train':len(rows),'fit':len(fit),'calibration':len(cal),'fit_groups':len({groups[i] for i in fit}),'calibration_groups':len({groups[i] for i in cal}),'fit_cleaned':len(cleaned),'unrestricted_programs':len(counts),'candidate_programs':len(bank),'candidate_fit_rows':len(eligible),'exclusions':dict(exclusions),'calibration_bounded_annotations':sum(ps[i] is not None for i in cal),'calibration_executable_annotations':sum(executions[i]['status']=='proposed' for i in cal),'calibration_clean_agreement':sum(ps[i] is not None and executions[i]['status']=='proposed' and grade(rows[i],executions[i])=='correct' for i in cal)})
    # Offline-only lexical retrieval: all cleaned FIT rows, no retrieval asset shipped.
    ws={i:set(words(rows[i]['Problem'])) for i in cleaned}
    wdf=collections.Counter(w for v in ws.values() for w in v)
    widf={w:math.log((1+len(cleaned))/(1+n))+1 for w,n in wdf.items()}
    postings=collections.defaultdict(list)
    for i in cleaned:
        norm=math.sqrt(sum(widf[w]**2 for w in ws[i]))
        for w in sorted(ws[i]):postings[w].append((i,widf[w]/norm))
    restrict=set(eligible)
    def predict(qs,tag):
        candidate=run(binary,['--propose-model',str(RUN/'model.json'),'--jsonl'],qs,RUN/(tag+'-candidate.jsonl'))
        choices=[[],[]];scores=[[],[]]
        for q in qs:
            qw=sorted(set(words(q)) & widf.keys());norm=math.sqrt(sum(widf[w]**2 for w in qw));sim=collections.defaultdict(float)
            if norm:
                for w in qw:
                    for i,v in postings[w]:sim[i]+=v*widf[w]/norm
            unrestricted=max(cleaned,key=lambda i:(sim[i],-i));restricted=max(eligible,key=lambda i:(sim[i],-i))
            for k,i in enumerate([unrestricted,restricted]):
                choices[k].append({'question':q,'program':ps[i]});scores[k].append(sim[i])
        outs={'candidate':candidate}
        for k,name in enumerate(['lexical','lexical_restricted']):
            out=run(executor,[],choices[k],RUN/(tag+'-'+name+'-execution.jsonl'))
            for o,s in zip(out,scores[k]):o['score']=s
            jsonl(RUN/(tag+'-'+name+'.jsonl'),out);outs[name]=out
        # All outputs/model/source hashes saved before scoring / label read.
        manifest(tag+'-prescore-manifest.json',[RUN/'model.json',RUN/'prefit-manifest.json',binary,executor,*[RUN/(tag+'-'+name+'.jsonl') for name in outs]])
        return outs
    calout=predict([rows[i]['Problem'] for i in cal],'calibration')
    labels=[rows[i] for i in cal]; thresholds={};calreport={}
    for name,out in calout.items():
        thresholds[name],front=threshold(labels,out);calreport[name]=report(labels,out,thresholds[name]);calreport[name]['best_precision_min30']=front
    dump(RUN/'thresholds.json',thresholds);dump(RUN/'calibration-report.json',calreport)
    print('FIT',json.dumps(json.loads((RUN/'fit-report.json').read_text())),flush=True)
    print('CALIBRATION',json.dumps(calreport),flush=True)
    # Only Problem is read before complete full predictions are saved.
    qs=pq.read_table(SOURCE/'validation.parquet',columns=['Problem']).column('Problem').to_pylist()
    order=sorted(range(len(qs)),key=lambda i:(hashlib.sha256(qs[i].encode()).hexdigest(),i));sample=order[:200]
    dump(RUN/'sample-indices.json',sample)
    sampleout=predict([qs[i] for i in sample],'sample')
    fullout=predict(qs,'full')
    # Separate scoring after predictions frozen. Never read DEV formulas/rationale/category.
    devlabels=pq.read_table(SOURCE/'validation.parquet',columns=['options','correct']).to_pylist()
    for tag,outs,labels in [('sample',sampleout,[devlabels[i] for i in sample]),('full',fullout,devlabels)]:
        rep={name:report(labels,out,thresholds[name]) for name,out in outs.items()};dump(RUN/(tag+'-report.json'),rep);print(tag.upper(),json.dumps(rep),flush=True)
    # Default solve compatibility, same executable and original frozen baseline.
    baseline=RUN.parent/'baseline/glassmind-quantities'
    syn=[]
    for p in [ROOT/'crates/quantities/dev/synthetic-dev.jsonl',ROOT/'crates/quantities/dev/extended-dev.jsonl']:
        syn += [json.loads(l)['question'] for l in p.read_text().splitlines()]
    assert len(syn)==3116
    compatibility={}
    for tag,questions in [('sample',[qs[i] for i in sample]),('full',qs),('synthetic',syn)]:
        old=RUN/('solve-'+tag+'-baseline.jsonl');new=RUN/('solve-'+tag+'-candidate.jsonl')
        run(baseline,['--jsonl'],questions,old);run(binary,['--jsonl'],questions,new)
        compatibility[tag]={'rows':len(questions),'identical':old.read_bytes()==new.read_bytes(),'baseline_sha256':sha(old),'candidate_sha256':sha(new)}
        assert compatibility[tag]['identical']
        if tag=='full':
            actuals=[json.loads(l) for l in new.read_text().splitlines()]
            grades=collections.Counter(compare(parse_gold(r['options'],r['correct'])[1],a)['grade'] for r,a in zip(devlabels,actuals))
            compatibility[tag]['original_metric']=dict(grades)
            compatibility[tag]['original_conservative_wrong']=len(actuals)-grades['correct']-grades['refused']
    dump(RUN/'compatibility.json',compatibility)
    manifest('final-manifest.json',[p for p in RUN.rglob('*') if p.is_file() and p.name!='final-manifest.json'])
if __name__=='__main__':main()
