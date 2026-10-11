"""Independent formal target contracts, frozen before worker inspection."""
from pathlib import Path
from fractions import Fraction
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
CONSTANTS=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
def case(name,op,left,right,pairs,prefix=None,n=3):return dict(name=name,input_count=n,prefix=prefix or [],op=op,left=left,right=right,expected_pairs=pairs)
def step(op,left,right):return dict(op=op,left=left,right=right)
cases=[
 case('add_commutes','add',0,1,[[0,1],[1,0]]),
 case('multiply_commutes','multiply',0,1,[[0,1],[1,0]]),
 case('subtract_ordered','subtract',0,1,[[0,1]]),
 case('divide_ordered','divide',0,1,[[0,1]]),
 case('equal_value_distinct_inputs','add',0,0,[[0,0]],n=2),
 case('input_one_not_constant_one','add',0,17,[[0,17],[17,0]],n=1),
 case('duplicate_prior_trees','subtract',36,37,[[36,36],[36,37],[37,36],[37,37]],prefix=[step('add',0,1),step('add',1,0)]),
 case('nested_commutation','multiply',36,37,[[36,37],[37,36]],prefix=[step('subtract',0,1),step('subtract',1,0)]),
 case('ordered_prior_different','add',0,36,[[0,36],[36,0]],prefix=[step('subtract',0,1),step('subtract',1,0)]),
 case('association_not_admitted','divide',38,39,[[38,39]],prefix=[step('add',0,1),step('add',1,2),step('add',36,2),step('add',0,37)]),
 case('cancellation_not_admitted','add',36,17,[[17,36],[36,17]],prefix=[step('divide',0,0)]),
 case('rational_constant_atom','multiply',0,33,[[0,33],[33,0]],n=1),
 case('zero_multiplication_not_collapsed','add',36,16,[[16,36],[36,16]],prefix=[step('multiply',0,16)]),
]
def node(op,a,b):
    if op in ['add','multiply']:a,b=sorted([a,b])
    return (op,a,b)
def targets(c):
    trees={i:('input',str(i)) for i in range(c['input_count'])}
    trees.update({16+i:('constant',str(Fraction(v))) for i,v in enumerate(CONSTANTS)})
    for i,s in enumerate(c['prefix']):trees[36+i]=node(s['op'],trees[s['left']],trees[s['right']])
    goal=node(c['op'],trees[c['left']],trees[c['right']])
    return [[a,b] for a in sorted(trees) for b in sorted(trees) if node(c['op'],trees[a],trees[b])==goal]
for c in cases:assert targets(c)==c['expected_pairs'],c['name']
invalid=[dict(name='missing_input',input_count=1,prefix=[],op='add',left=1,right=0),dict(name='forward_step',input_count=1,prefix=[],op='add',left=36,right=0),dict(name='invalid_constant',input_count=1,prefix=[],op='add',left=99,right=0)]
plan='''# Parent symbolic-supervision contracts

Freeze before worker inspection: thirteen formal pair-set contracts, three invalid
reference contracts, and native execution at distinct, negative, zero and equal
input assignments. Only Add/Multiply tree-child commutation is admitted. Numeric
coincidence, association, cancellation and equality of different input slots never
define labels. Previous-step expressions may be reused only when canonical trees
are identical. All expected pairs are authored here and independently enumerated.

Original native executor establishes behavior and refusal before the experiment.
Later check every worker TRAIN target pair set against this separate oracle, all
source/split hashes, untouched operation weights, matched training configuration,
gradient/probability calculation and all original/final numeric comparisons. A
TRAIN gate is continuation evidence, not a semantic or production acceptance.
No worker source/results inspected while it runs; archive before inspection.
'''
(HERE/'PLAN.md').write_text(plan)
(HERE/'cases.json').write_text(json.dumps(dict(valid=cases,invalid=invalid),indent=2)+'\n')

def arg(i):
    if i<16:return dict(kind='input',index=i)
    if i<36:return dict(kind='constant',index=i-16)
    return dict(kind='step',index=i-36)
def program(c,a,b):
    return {'steps':[dict(op=s['op'],left=arg(s['left']),right=arg(s['right'])) for s in c['prefix']+[step(c['op'],a,b)]]}
requests=[];groups=[]
for c in cases:
    for values in [['7','3','2'],['-4','9','0'],['0','0','0'],['7','7','7'],['0.25','-3.6','1.6']]:
        values=values[:c['input_count']];question='; '.join('quantity '+x for x in values)
        start=len(requests)
        for a,b in c['expected_pairs']:requests.append(dict(question=question,program=program(c,a,b)))
        groups.append(dict(name=c['name'],values=values,start=start,end=len(requests)))
binary=ROOT/'groundingaudit-source/equation_executor'
run=subprocess.run([str(binary)],input=''.join(json.dumps(r)+'\n' for r in requests),text=True,capture_output=True,check=True,timeout=60)
responses=[json.loads(s) for s in run.stdout.splitlines()];assert len(responses)==len(requests)
for g in groups:
    outputs=responses[g['start']:g['end']];first=outputs[0]
    assert all((r['status'],r.get('value'),r.get('reason'))==(first['status'],first.get('value'),first.get('reason')) for r in outputs),g
    if first['status']=='refusal':assert g['name'] in ['divide_ordered','cancellation_not_admitted','association_not_admitted'],(g,first)
(HERE/'native-baseline.json').write_text(json.dumps(dict(groups=groups,responses=responses,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest()),indent=2)+'\n')
(HERE/'freeze.json').write_text(json.dumps({n:hashlib.sha256((HERE/n).read_bytes()).hexdigest() for n in ['PLAN.md','cases.json','native-baseline.json','freeze.py']},indent=2)+'\n')
print(json.dumps(dict(formal_cases=len(cases),invalid_cases=len(invalid),native_groups=len(groups),native_executions=len(requests))))
