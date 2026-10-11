#!/usr/bin/env python3
"""Offline single configuration; predictions frozen before separate label scoring."""
import collections, hashlib, importlib.util, json, os, shutil, subprocess, sys
from pathlib import Path
import pyarrow.parquet as pq
ROOT=Path(__file__).resolve().parents[3]
OLD=Path('/home/gsc/.cache/glassmind/continuous-20261010/equationproposals-run/frozen-candidate')
SOURCE=Path('/home/gsc/.cache/glassmind/continuous-20261010/equation-source')
RUN=Path(os.environ['SEQUENCE_RUN']);TARGET=Path(os.environ['CARGO_TARGET_DIR'])
os.environ['PROPOSAL_RUN']=str(RUN);os.environ['PROPOSAL_SOURCE']=str(SOURCE)
sys.path.insert(0,str(ROOT/'tools/quantities/typed-label-metric'))
spec=importlib.util.spec_from_file_location('retained',OLD/'submission/experiment.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
# Unchanged metric implementation and split/parser/selection functions from retained worker.
grade,report,threshold,group,program,key=old.grade,old.report,old.threshold,old.group,old.program,old.key
sha,dump,jsonl=old.sha,old.dump,old.jsonl
SCOPE=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
def run(exe,args,rows,path):
    data=''.join(json.dumps(x,separators=(',',':'))+'\n' for x in rows)
    p=subprocess.run(SCOPE+[str(exe),*map(str,args)],input=data,text=True,capture_output=True,timeout=610)
    path.with_suffix('.stderr').write_text(p.stderr);path.write_text(p.stdout)
    p.check_returncode();out=[json.loads(x) for x in p.stdout.splitlines()];assert len(out)==len(rows);return out
def manifest(name,paths):dump(RUN/name,{str(p):{'sha256':sha(p),'bytes':p.stat().st_size} for p in paths})
def verify_old(name,manifest_name):
    p=OLD/name;m=json.loads((OLD/manifest_name).read_text());v=m[str(p)];assert sha(p)==v['sha256'];assert p.stat().st_size==v['bytes']
def main():
    RUN.mkdir(parents=True,exist_ok=True);assert not (RUN/'model.json').exists()
    for name in ['train-executions.jsonl','split.json']:
        verify_old(name,'final-manifest.json')
    for tag in ['calibration','sample','full']:
        for baseline in ['lexical','lexical_restricted']:verify_old(tag+'-'+baseline+'.jsonl',tag+'-prescore-manifest.json')
    for name in ['solve-sample-baseline.jsonl','solve-full-baseline.jsonl','solve-synthetic-baseline.jsonl']:verify_old(name,'final-manifest.json')
    binary=RUN/'equation_sequence';trainer=RUN/'equation_fit';executor=RUN/'equation_executor';solver=RUN/'glassmind-quantities'
    for name in ['equation_sequence','equation_fit','equation_executor']:shutil.copy2(TARGET/'release/examples'/name,RUN/name)
    shutil.copy2(TARGET/'release/glassmind-quantities',solver)
    frozen=RUN/'source'
    files=list((ROOT/'crates/quantities/src').glob('*.rs'))+list((ROOT/'crates/quantities/examples').glob('equation*.rs'))+[ROOT/'crates/quantities/tests/sequence.rs',ROOT/'crates/quantities/Cargo.toml',ROOT/'Cargo.lock']+list(Path(__file__).parent.glob('*.py'))+[Path(__file__).parent/'PLAN.md']
    for p in files:
        dest=frozen/p.relative_to(ROOT);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest)
    manifest('prefit-manifest.json',[*filter(Path.is_file,frozen.rglob('*')),binary,trainer,executor,solver,SOURCE/'train.parquet',SOURCE/'validation.parquet',SOURCE/'source-card.md'])
    rows=pq.read_table(SOURCE/'train.parquet',columns=['Problem','linear_formula','options','correct']).to_pylist()
    split=json.loads((OLD/'split.json').read_text());groups=[group(r['Problem']) for r in rows]
    assert groups==split['group_hashes'];fit=[i for i,g in enumerate(groups) if int(g,16)%5];cal=[i for i,g in enumerate(groups) if not int(g,16)%5]
    assert fit==split['fit'] and cal==split['calibration'];dump(RUN/'split.json',split)
    ps=[program(r['linear_formula']) for r in rows]
    executions=run(executor,[],[{'question':r['Problem'],'program':p} for r,p in zip(rows,ps)],RUN/'train-executions.jsonl')
    retained=[json.loads(l) for l in (OLD/'train-executions.jsonl').read_text().splitlines()]
    assert executions==retained,'executor mismatch before fit'
    cleaned=[i for i in fit if ps[i] is not None and executions[i]['status']=='proposed' and grade(rows[i],executions[i])=='correct']
    fit_programs=set(key(ps[i]) for i in cleaned);assert len(cleaned)==4590 and len(fit_programs)==1565
    dump(RUN/'audit.json',{'executor_rows_identical':len(executions),'group_hashes_identical':len(groups),'cleaned_fit':len(cleaned),'structures':len(fit_programs),'calibration':len(cal),'baseline_hashes_verified':True})
    fitrows=[{'question':rows[i]['Problem'],'program':ps[i]} for i in cleaned];jsonl(RUN/'fit-input.jsonl',fitrows)
    provenance='MathQA TRAIN '+sha(SOURCE/'train.parquet')+'; PLAN '+sha(Path(__file__).parent/'PLAN.md')+'; all4590 grouped FIT; linear operation/left/right v1'
    with (RUN/'fit-input.jsonl').open() as inp,(RUN/'fit-report.json').open('w') as out,(RUN/'fit.stderr').open('w') as err:
        p=subprocess.run(SCOPE+[str(trainer),str(RUN/'model.json'),provenance],stdin=inp,stdout=out,stderr=err,timeout=610);p.check_returncode()
    assert (RUN/'model.json').stat().st_size<=32_000_000
    manifest('postfit-manifest.json',[RUN/'model.json',RUN/'fit-report.json',RUN/'prefit-manifest.json'])
    print('FIT', (RUN/'fit-report.json').read_text(),flush=True)
    def predict(qs,tag):
        jsonl(RUN/(tag+'-questions.jsonl'),qs)
        out=run(binary,[RUN/'model.json'],qs,RUN/(tag+'-candidate.jsonl'))
        manifest(tag+'-prescore-manifest.json',[RUN/'model.json',RUN/'prefit-manifest.json',binary,executor,RUN/(tag+'-questions.jsonl'),RUN/(tag+'-candidate.jsonl')])
        return out
    calout=predict([rows[i]['Problem'] for i in cal],'calibration');labels=[rows[i] for i in cal]
    t,front=threshold(labels,calout);dump(RUN/'thresholds.json',{'candidate':t,'lexical':None,'lexical_restricted':None})
    cr=report(labels,calout,t);cr['best_precision_min30']=front;dump(RUN/'calibration-report.json',cr);print('CAL',json.dumps(cr),flush=True)
    qs=pq.read_table(SOURCE/'validation.parquet',columns=['Problem']).column('Problem').to_pylist()
    sample=sorted(range(len(qs)),key=lambda i:(hashlib.sha256(qs[i].encode()).hexdigest(),i))[:200]
    assert sample==json.loads((OLD/'sample-indices.json').read_text());dump(RUN/'sample-indices.json',sample)
    sampleout=predict([qs[i] for i in sample],'sample')
    assert all(o.get('semantics')=='unverified' and o.get('model_sha256')==sha(RUN/'model.json') for o in sampleout if o['status']=='proposed')
    fullout=predict(qs,'full')
    # Only after complete frozen outputs: read DEV options/correct, never formulas.
    devlabels=pq.read_table(SOURCE/'validation.parquet',columns=['options','correct']).to_pylist()
    for tag,out,labels in [('sample',sampleout,[devlabels[i] for i in sample]),('full',fullout,devlabels)]:
        rep={'candidate':report(labels,out,t)}
        for b in ['lexical','lexical_restricted']:
            baseline=[json.loads(l) for l in (OLD/(tag+'-'+b+'.jsonl')).read_text().splitlines()]
            assert len(baseline)==len(out);rep[b]=report(labels,baseline,None)
        proposed=[o for o in out if o['status']=='proposed'];novel=[o for o in proposed if key(o['program']) not in fit_programs]
        rep['composition']={'proposed':len(proposed),'distinct_programs':len({key(o['program']) for o in proposed}),'novel_rows':len(novel),'distinct_novel_programs':len({key(o['program']) for o in novel}),'max_steps':max((len(o['program']['steps']) for o in proposed),default=0),'beam':1,'novel_correct':sum(grade(r,o)=='correct' for r,o in zip(labels,out) if o['status']=='proposed' and key(o['program']) not in fit_programs)}
        dump(RUN/(tag+'-report.json'),rep);print(tag.upper(),json.dumps(rep),flush=True)
    syn=[]
    for name in ['synthetic-dev.jsonl','extended-dev.jsonl']:syn.extend(json.loads(l)['question'] for l in (ROOT/'crates/quantities/dev'/name).read_text().splitlines())
    assert len(syn)==3116;compat={}
    for tag,questions in [('sample',[qs[i] for i in sample]),('full',qs),('synthetic',syn)]:
        p=RUN/('solve-'+tag+'-candidate.jsonl');run(solver,['--jsonl'],questions,p)
        original=OLD/('solve-'+tag+'-baseline.jsonl');assert p.read_bytes()==original.read_bytes()
        compat[tag]={'rows':len(questions),'identical':True,'sha256':sha(p)}
    dump(RUN/'compatibility.json',compat)
    rep=json.loads((RUN/'full-report.json').read_text());c=rep['candidate'];passed=(c['raw']['correct']>=688 and t is not None and c['selected_correct']>=50 and c['selected_precision']>=.8 and c['selected_correct']>=10)
    dump(RUN/'decision.json',{'accepted':passed,'raw_gate':c['raw']['correct']>=688,'calibration_gate':t is not None,'selected_gate':bool(c['selected_precision'] is not None and c['selected_correct']>=50 and c['selected_precision']>=.8),'no_parent_checks_claimed':True})
    manifest('final-manifest.json',[p for p in RUN.rglob('*') if p.is_file() and p.name!='final-manifest.json'])
if __name__=='__main__':main()
