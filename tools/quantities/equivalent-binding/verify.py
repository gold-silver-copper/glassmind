"""Independent symbolic target reconstruction and native-output integrity checks."""
import collections,hashlib,json,os,struct
from fractions import Fraction
from pathlib import Path
ROOT=Path.cwd();S=ROOT.parent/'groundingaudit-source';R=Path(os.environ.get('EQUIV_RUN',str(ROOT.parent/'equivbinding-run/experiment')))
read=lambda p:[json.loads(l) for l in p.read_text().splitlines()]
const=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
def atom(i,prior,n):
    if 0<=i<n:return ('input',i)
    if 16<=i<36:
        q=Fraction(const[i-16]);return ('constant',q.numerator,q.denominator)
    if 36<=i<36+len(prior):return prior[i-36]
    raise ValueError('illegal reference')
def aid(a):return a['index']+{'input':0,'constant':16,'step':36}[a['kind']]
def tree(op,l,r):
    # repr ordering intentionally differs from Rust enum ordering; canonical equality
    # is nevertheless the same quotient by commutation of immediate children only.
    if op in ('add','multiply'):l,r=sorted([l,r],key=repr)
    return (op,l,r)
clean=read(R/'clean.jsonl');expect={};expanded=collections.Counter();total=collections.Counter();maxpairs=0
for c in clean:
    # All input counts come from retained original native bindings, never numeric values.
    expect[c['index']]=[]
executions=read(S/'train-executions.jsonl')
for c in clean:
    n=len(executions[c['index']]['numbers']);prior=[]
    for step in c['program']['steps']:
        actions=list(range(n))+list(range(16,36))+list(range(36,36+len(prior)))
        at={i:atom(i,prior,n) for i in actions};op=step['op'];gl,gr=aid(step['left']),aid(step['right'])
        gold=tree(op,at[gl],at[gr]);valid=[(l,r) for l in actions for r in actions if tree(op,at[l],at[r])==gold]
        assert (gl,gr) in valid
        expect[c['index']].append(valid);prior.append(gold)
        expanded[c['split']]+=len(valid)>1;total[c['split']]+=1;maxpairs=max(maxpairs,len(valid))
for arm in ['original','A','B']:
    records=read(R/(arm+'-full.jsonl'));assert len(records)==5772
    for c,r in zip(clean,records,strict=True):
        assert (c['index'],c['split'])==(r['index'],r['split'])
        for e,pairs in zip(r['diagnostic']['teacher'],expect[c['index']],strict=True):
            assert [tuple(p) for p in e['valid_pairs']]==pairs
            assert e['left_equiv']==any(l==e['left'] for l,_ in pairs)
            assert e['right_equiv']==((e['gold_left'],e['right_gold_left']) in pairs)
            assert e['pair_equiv']==((e['left'],e['right_pred_left']) in pairs)
    weights=json.loads((S/'model.json' if arm=='original' else R/(arm+'-model.json')).read_text())
    assert set(weights)=={'schema','provenance','weights'}
# Exact included-source check, with no changed original lines.
proposal=(S/'rejected-source/crates/quantities/src/proposal.rs').read_bytes()
assert (R/'helper/src/proposal.rs').read_bytes()==proposal+b'\ninclude!("instrument.rs");\ninclude!("checks.rs");\n'
assert (R/'helper/src/rational.rs').read_bytes()==(S/'rejected-source/crates/quantities/src/rational.rs').read_bytes()
for f in ['instrument.rs','checks.rs','main.rs']:
    assert (R/'helper/src'/f).read_bytes()==(ROOT/'tools/quantities/equivalent-binding'/f).read_bytes()==(R/'prefit'/f).read_bytes()
for f in json.loads((S/'manifest.json').read_text())['files']:
    assert hashlib.sha256((S/f['file']).read_bytes()).hexdigest()==f['sha256']
report=dict(passed=True,independent_targets_all_arms=True,steps=dict(total),expanded_steps=dict(expanded),max_pairs=maxpairs,all_original_source_bytes_preserved=True,all_supplied_hashes_verified_again=True)
(R/'independent-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
