"""One fixed TRAIN-only matched experiment. No checkpoint selection or thresholds."""
import collections, hashlib, json, os, shutil, struct, subprocess, sys, time
from pathlib import Path
ROOT=Path.cwd();S=ROOT.parent/'groundingaudit-source';R=Path(os.environ.get('EQUIV_RUN',str(ROOT.parent/'equivbinding-run/experiment')))
O=ROOT/'tools/quantities/equivalent-binding';BIN=Path(os.environ['CARGO_TARGET_DIR'])/'release/equivalent-binding-helper'
sys.path.insert(0,str(R));from metric import parse_gold,compare
read=lambda p:[json.loads(l) for l in p.read_text().splitlines()]
def dump(p,v):p.write_text(json.dumps(v,sort_keys=True,indent=2)+'\n')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def jl(p,v):p.write_text(''.join(json.dumps(x,separators=(',',':'))+'\n' for x in v))
def run(tag,cmd,inp):
    full=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600',*map(str,cmd)]
    with (R/'commands.jsonl').open('a') as f:f.write(json.dumps(dict(tag=tag,argv=full,input=str(inp),TMPDIR=os.environ['TMPDIR'],CARGO_TARGET_DIR=os.environ['CARGO_TARGET_DIR']))+'\n')
    t=time.monotonic()
    with inp.open('rb') as i,(R/(tag+'.jsonl')).open('wb') as o,(R/(tag+'.stderr')).open('wb') as e:
        p=subprocess.run(full,stdin=i,stdout=o,stderr=e,timeout=620)
    with (R/'timings.jsonl').open('a') as f:f.write(json.dumps(dict(tag=tag,seconds=time.monotonic()-t,returncode=p.returncode))+'\n')
    if p.returncode:raise RuntimeError(tag+' failed; no partial artifact selected')
    return read(R/(tag+'.jsonl'))
def grade(row,out):
    actual={'status':'answer','value':out['value'],'unit':None} if out['status']=='proposed' else {'status':'refusal'}
    return compare(parse_gold(row['options'],row['correct'])[1],actual)['grade']
def op_hash(path):
    m=json.loads(path.read_text());return hashlib.sha256(b''.join(struct.pack('<f',w) for w in m['weights'][0])).hexdigest()
clean=read(R/'clean.jsonl'); assert len(clean)==5772
assert sha(O/'PLAN.md')==sha(R/'PLAN.md')
original=run('baseline200-original',[S/'equation_sequence',S/'model.json'],R/'replay200-input.jsonl')
helper=run('baseline200-helper',[BIN,S/'model.json','--free'],R/'replay200-input.jsonl')
assert original==helper==read(R/'replay200-expected.jsonl')
dump(R/'baseline200-check.json',dict(rows=200,original_exact=True,helper_exact=True))
# Independent Fraction arithmetic contracts through original executor.
from fractions import Fraction
contracts=[];expect=[]
def arg(kind,i):return dict(kind=kind,index=i)
def st(op,l,r):return dict(op=op,left=l,right=r)
a=arg('input',0);b=arg('input',1);p=arg('step',0)
for x,y in [('2.5','-3'),('-1.25','4'),('0','-2'),('3','0')]:
    for op in ['add','multiply','duplicate','division']:
        for swap in [False,True]:
            X,Y=Fraction(x),Fraction(y)
            if op in ['add','multiply']:
                steps=[st(op,b,a) if swap else st(op,a,b)];v=X+Y if op=='add' else X*Y
            elif op=='duplicate':
                steps=[st('add',a,b),st('add',b,a),st('subtract',arg('step',int(swap)),a)];v=Y
            else:
                steps=[st('divide',a,b),st('add',a,p) if swap else st('add',p,a)];v=X/Y+X if Y else None
            contracts.append(dict(question=f'{x} and {y}',program=dict(steps=steps)));expect.append(None if v is None else str(v))
jl(R/'independent-contracts-input.jsonl',contracts)
executed=run('independent-contracts-native',[S/'equation_executor'],R/'independent-contracts-input.jsonl')
for e,v in zip(executed,expect,strict=True):assert (e.get('value')==v if v is not None else e['status']=='refusal'),(e,v)
dump(R/'independent-contracts-check.json',dict(cases=len(expect),passed=True,expected=expect,definition='Python Fraction independently checks native transformations; never defines targets'))
results={}; records={}
def evaluate(name,model):
    # Fixed first200 clean CAL diagnostic sample before full affected TRAIN subsets.
    sample=[r for r in clean if r['split']=='CAL'][:200];jl(R/'sample-input.jsonl',sample)
    sampled=run(name+'-sample',[BIN,model,'--diagnostics'],R/'sample-input.jsonl')
    full=run(name+'-full',[BIN,model,'--diagnostics'],R/'clean.jsonl')
    assert sampled==[r for r in full if r['split']=='CAL'][:200]
    assert len(full)==len(clean)
    totals={s:collections.Counter() for s in ['FIT','CAL']}
    for c,r in zip(clean,full,strict=True):
        assert (r['index'],r['split'])==(c['index'],c['split']);t=totals[c['split']];t['rows']+=1
        d=r['diagnostic'];events=d['teacher'];t['operand_denominator']+=len(events)
        for k in ['forced_stop','forced_horizon']:t[k]+=d[k]
        for mode,pred in [('free',r['free']),('oracle',d['oracle'])]:
            t[mode+'_exactAST']+=pred.get('program')==c['program'];t[mode+'_'+grade(c,pred)]+=1
        for k in ['left_exact','right_exact','left_equiv','right_equiv','pair_equiv']:
            t[k]+=sum(e[k] for e in events);t['whole_'+k]+=all(e[k] for e in events)
        t['whole_exact_grounding']+=all(e['left_exact'] and e['right_exact'] for e in events)
        t['whole_equiv_gold_context_grounding']+=all(e['left_equiv'] and e['right_equiv'] for e in events)
        t['op_nonstop_exact']+=sum(e['op']==e['op_pred'] for e in events)
    for split,n,den in [('FIT',4590,15753),('CAL',1182,4138)]:
        t=totals[split];assert t['rows']==n and t['operand_denominator']==den
        for mode in ['free','oracle']:
            assert sum(t[mode+'_'+g] for g in ['correct','refused','wrong','unscorable'])==n
            assert t[mode+'_unscorable']==0
    results[name]=totals;records[name]=full
    dump(R/(name+'-summary.json'),totals)
    print(name,json.dumps(totals),flush=True)
evaluate('original',S/'model.json')
b=results['original']['CAL'];assert (b['free_correct'],b['oracle_correct'],b['left_exact'],b['right_exact'])==(143,221,2197,2499)
f=results['original']['FIT'];assert (f['free_correct'],f['oracle_correct'],f['left_exact'],f['right_exact'])==(1863,2334,12886,12857)
# Original clean CAL free outputs must also exactly match retained original full CAL.
old=read(S/'calibration-candidate.jsonl');split=json.loads((S/'split.json').read_text());mapping=dict(zip(split['calibration'],old,strict=True))
assert all(r['free']==mapping[r['index']] for r in records['original'] if r['split']=='CAL')
pre=R/'prefit';pre.mkdir()
for pth in O.iterdir():
    if pth.is_file():shutil.copy2(pth,pre/pth.name)
for name in ['corpus-frozen.json','baseline200-check.json','original-summary.json','checks.log','independent-contracts-check.json']:
    shutil.copy2(R/name,pre/name)
dump(R/'prefit-manifest.json',{str(p.relative_to(R)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in [*pre.iterdir(),R/'membership.jsonl',R/'clean.jsonl',S/'model.json'] if p.is_relative_to(R)})
for arm in ['A','B']:
    model=R/(arm+'-model.json')
    run(arm+'-training',[BIN,S/'model.json','--fit-'+arm.lower(),model],R/'fit-input.jsonl')
    assert model.stat().st_size<=16*1024*1024 and op_hash(model)==op_hash(S/'model.json')
    evaluate(arm,model)
transitions=[]
for arm in ['A','B']:
    for c,old,new in zip(clean,records['original'],records[arm],strict=True):
        for mode in ['free','oracle']:
            x=old['free'] if mode=='free' else old['diagnostic']['oracle'];y=new['free'] if mode=='free' else new['diagnostic']['oracle']
            before,after=grade(c,x),grade(c,y)
            if (before=='correct' and after!='correct') or (before=='refused' and after=='wrong'):
                transitions.append(dict(arm=arm,index=c['index'],split=c['split'],mode=mode,before=before,after=after))
jl(R/'losses-and-refusal-to-wrong.jsonl',transitions)
A=results['A']['CAL'];B=results['B']['CAL'];base=results['original']['CAL']
gates={'free_min168':B['free_correct']>=168,'oracle_min251':B['oracle_correct']>=251,'free_above_A':B['free_correct']>A['free_correct'],'oracle_above_A':B['oracle_correct']>A['oracle_correct']}
for k in ['left_equiv','right_equiv']:
    gates[k+'_gain_over_original']=B[k]-base[k]>=125
    gates[k+'_gain_over_A']=B[k]-A[k]>=125
summary=dict(results=results,gates=gates,accepted_for_parent_continuation=all(gates.values()),decision='PASS_FOR_REVIEW_ONLY' if all(gates.values()) else 'REJECT',op_hashes={arm:op_hash(R/(arm+'-model.json')) for arm in ['A','B']},original_op_hash=op_hash(S/'model.json'),transition_counts=dict(collections.Counter(f"{r['arm']}/{r['split']}/{r['mode']}/{r['before']}->{r['after']}" for r in transitions)),selected_answers=0)
dump(R/'results.json',summary);print(json.dumps(summary,indent=2),flush=True)
