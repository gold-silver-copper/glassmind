"""Authored protocol contracts, never exposed to the annotation teacher."""
import copy,json,re,sys
from pathlib import Path
from validate import CONSTANTS,execute,load,unique_keys,validate


def row(name,question,answer,final):
    numbers=[dict(start=len(question[:m.start()].encode()),end=len(question[:m.end()].encode()),literal=m[0],value=m[0]) for m in re.finditer(r'[0-9]+',question)]
    return dict(id=name,question=question,answer=answer,final_answer=final,numbers=numbers)


def cite(r,field='question'):
    value=r[field];return [dict(field=field,start=0,end=len(value.encode()),quote=value)]


def quantity(r,mentions,meaning,unit,coreference=''):
    return dict(mentions=mentions,meaning=meaning,unit=unit,coreference=coreference,evidence=cite(r))


def operand(r,kind,index,role):
    return dict(source=dict(kind=kind,index=index),role=role,evidence=cite(r))


def step(r,op,left,right,value,meaning,unit):
    return dict(op=op,left=operand(r,*left),right=operand(r,*right),value=value,meaning=meaning,unit=unit,evidence=cite(r))


def annotation(r,quantities,steps):
    return dict(id=r['id'],status='resolved',reason='Authored mechanically valid graph; semantic fixture reviewed separately.',evidence=cite(r),rationale_issues=[],quantities=quantities,steps=steps)


a=row('coreference','Élodie has 20 dollars. Her 20 dollars pay for 4 notebooks at 3 dollars each. How much remains?','Notebooks cost12dollars;8dollars remain.','8')
b=row('missing-conversion','Rin rode 30 minutes on Tuesday and 30 minutes on Thursday. How many hours altogether?','The rides take <<30+30=60>>60 minutes, or1hour.','1')
c=row('semantic-distractor','Mira packed 7 green boxes and 8 red boxes. Each box held 9 pencils. There were 15 crates nearby. How many pencils were in the boxes?','There are15boxes and135pencils.','135')
d=row('bad-rationale','A basket holds 9 apples. Lee removes 2. How many remain?','The source incorrectly adds <<9+2=11>>11.','11')
qa=[quantity(a,[0,1],'Same original budget, repeated with her','dollars','Her20dollars refers back to the originally stated20-dollar budget.'),quantity(a,[2],'Purchased notebooks','notebooks'),quantity(a,[3],'Notebook unit price','dollars/notebook')]
sa=[step(a,'multiply',('quantity',1,'Notebook count'),('quantity',2,'Unit price'),'12','Purchase cost','dollars'),step(a,'subtract',('quantity',0,'Original budget'),('step',0,'Computed purchase cost'),'8','Money remaining','dollars')]
qb=[quantity(b,[0],'Tuesday ride duration','minutes'),quantity(b,[1],'Thursday ride duration','minutes')]
sb=[step(b,'add',('quantity',0,'Tuesday minutes'),('quantity',1,'Thursday minutes'),'60','Total riding duration','minutes'),step(b,'divide',('step',0,'Combined riding minutes'),('constant',11,'60minutes/hour unit conversion'),'1','Total riding duration','hours')]
qc=[quantity(c,[0],'Green boxes','boxes'),quantity(c,[1],'Red boxes','boxes'),quantity(c,[2],'Pencils per box','pencils/box')]
sc=[step(c,'add',('quantity',0,'Green box count'),('quantity',1,'Red box count'),'15','Combined box count','boxes'),step(c,'multiply',('step',0,'Computed total boxes'),('quantity',2,'Pencils per box'),'135','Pencils in all boxes','pencils')]
refusal=dict(id=d['id'],status='label_error',reason='Removal is subtraction; reference final11 contradicts the question.',evidence=cite(d),rationale_issues=['Reference adds instead of subtracting.'],quantities=[],steps=[])
batch=dict(constants=CONSTANTS,rows=[a,b,c,d]);base=dict(schema=2,rows=[annotation(a,qa,sa),annotation(b,qb,sb),annotation(c,qc,sc),refusal]);results=[]


def check(name,mutate=None,accept=False,change_batch=None,inspect=None):
    x,y=copy.deepcopy(base),copy.deepcopy(batch)
    if mutate:mutate(x)
    if change_batch:change_batch(y)
    try:result=validate(y,x)
    except (ValueError,TypeError,IndexError,KeyError) as error:
        assert not accept,(name,str(error));results.append(dict(name=name,rejected=True,reason=str(error)));return
    assert accept,name
    assert result['semantic_approval'] is False
    if inspect:inspect(result)
    results.append(dict(name=name,rejected=False,accepted=result['mechanically_accepted']))


def representative(result):
    assert result['rows'][0]['program']['steps'][1]['left']==dict(kind='input',index=0)


check('complete-reconstruction-coreference-and-honest-refusal',accept=True,inspect=representative)
check('unordered-coreference-canonical-representative',lambda x:x['rows'][0]['quantities'][0].update(mentions=[1,0]),accept=True,inspect=representative)
check('extra-top-field',lambda x:x.update(hidden_gold=True))
check('wrong-schema',lambda x:x.update(schema=True))
check('missing-row',lambda x:x['rows'].pop())
check('duplicate-row',lambda x:x['rows'].__setitem__(1,copy.deepcopy(x['rows'][0])))
check('unknown-row',lambda x:x['rows'][0].update(id='other'))
check('unknown-status',lambda x:x['rows'][0].update(status='approved'))
check('nonacceptance-with-graph',lambda x:x['rows'][0].update(status='ambiguous'))
check('resolved-empty-graph',lambda x:x['rows'][0].update(steps=[]))
check('too-many-steps',lambda x:x['rows'][0]['steps'].extend(copy.deepcopy(x['rows'][0]['steps'])*3))
check('too-many-native-inputs',change_batch=lambda y:y['rows'][0]['numbers'].extend([copy.deepcopy(y['rows'][0]['numbers'][0])]*13))
check('native-constants-unchanged',change_batch=lambda y:y.update(constants=CONSTANTS+['7']))
check('extra-quantity-field',lambda x:x['rows'][0]['quantities'][0].update(score=1))
check('duplicate-mention',lambda x:x['rows'][0]['quantities'][0].update(mentions=[0,0]))
check('overlapping-quantity-groups',lambda x:x['rows'][0]['quantities'][1].update(mentions=[1,2],coreference='Claimed same quantity'))
check('boolean-mention',lambda x:x['rows'][0]['quantities'][0].update(mentions=[False,1]))
check('out-of-range-mention',lambda x:x['rows'][0]['quantities'][0].update(mentions=[0,8]))
check('unequal-coreference-values',lambda x:x['rows'][0]['quantities'][0].update(mentions=[0,2]),change_batch=lambda y:None)
check('missing-coreference-explanation',lambda x:x['rows'][0]['quantities'][0].update(coreference=''))
check('missing-quantity-unit',lambda x:x['rows'][0]['quantities'][0].update(unit=''))
check('uncited-group-mention',lambda x:x['rows'][0]['quantities'][0].update(evidence=[dict(field='question',start=a['numbers'][0]['start'],end=a['numbers'][0]['end'],quote='20')]))
check('quantity-rationale-only',lambda x:x['rows'][0]['quantities'][0].update(evidence=cite(a,'answer')))
check('split-utf8-codepoint',lambda x:x['rows'][0]['evidence'][0].update(start=1))
check('boolean-evidence-offset',lambda x:x['rows'][0]['evidence'][0].update(start=False))
check('invented-quote',lambda x:x['rows'][0]['evidence'][0].update(quote='Not in question'))
check('negative-evidence-offset',lambda x:x['rows'][0]['evidence'][0].update(start=-1))
check('extra-evidence-key',lambda x:x['rows'][0]['evidence'][0].update(confidence=1))
check('future-step',lambda x:x['rows'][0]['steps'][1]['right']['source'].update(index=1))
check('negative-step',lambda x:x['rows'][0]['steps'][1]['right']['source'].update(index=-1))
check('boolean-source',lambda x:x['rows'][0]['steps'][0]['left']['source'].update(index=True))
check('unknown-source-kind',lambda x:x['rows'][0]['steps'][0]['left']['source'].update(kind='answer'))
check('unknown-quantity',lambda x:x['rows'][0]['steps'][0]['left']['source'].update(index=10))
check('new-native-constant',lambda x:x['rows'][1]['steps'][1]['right']['source'].update(index=20))
check('constant-rationale-only',lambda x:x['rows'][1]['steps'][1]['right'].update(evidence=cite(b,'answer')))
check('operand-misses-quantity',lambda x:x['rows'][0]['steps'][0]['left'].update(evidence=[dict(field='question',start=a['numbers'][3]['start'],end=a['numbers'][3]['end'],quote='3')]))
check('empty-operand-role',lambda x:x['rows'][0]['steps'][0]['left'].update(role=''))
check('extra-operand-field',lambda x:x['rows'][0]['steps'][0]['left'].update(probability=1))
check('unknown-operation',lambda x:x['rows'][0]['steps'][0].update(op='power'))
check('missing-step-unit',lambda x:x['rows'][0]['steps'][0].update(unit=''))
check('step-rationale-only',lambda x:x['rows'][0]['steps'][0].update(evidence=cite(a,'answer')))
check('wrong-intermediate',lambda x:x['rows'][0]['steps'][0].update(value='13'))
check('nonfinite-value',lambda x:x['rows'][0]['steps'][0].update(value='NaN'))
check('fraction-zero-denominator',lambda x:x['rows'][0]['steps'][0].update(value='12/0'))
check('changed-source-final',change_batch=lambda y:y['rows'][0].update(final_answer='9'))
check('zero-divisor',lambda x:x['rows'][1]['steps'][1]['right']['source'].update(index=0))
check('unused-quantity',lambda x:x['rows'][2]['quantities'].append(quantity(c,[3],'Nearby crates','crates')))


def dead(x):
    x['rows'][0]['steps'].append(step(a,'add',('constant',4,'Invented four'),('constant',4,'Invented four'),'8','Fake result','dollars'))


check('dead-calculation',dead)
# Same numeric answer, wrong source. Delete the unused valid computation and its
# now-unused quantities so dead-code checks cannot falsely appear to prove meaning.
def distractor(x):
    r=x['rows'][2]
    r['quantities']=[quantity(c,[3],'Claimed relevant container count','containers'),quantity(c,[2],'Pencils per container','pencils/container')]
    r['steps']=[step(c,'multiply',('quantity',0,'Nearby crates wrongly treated as boxes'),('quantity',1,'Pencils per box wrongly paired with crates'),'135','Claimed pencils','pencils')]


check('equal-value-distractor-needs-semantic-review',distractor,accept=True)
# Numeric agreement and citations cannot establish that two equal-duration events
# refer to the same event. This intentionally false coreference remains mechanical.
def false_coreference(x):
    r=x['rows'][1]
    r['quantities']=[quantity(b,[0,1],'Claimed repeated duration','minutes','Incorrectly claims Tuesday and Thursday are the same ride.')]
    r['steps']=[step(b,'multiply',('quantity',0,'Claimed shared duration'),('constant',2,'Two rides'),'60','Total minutes','minutes'),step(b,'divide',('step',0,'Claimed combined minutes'),('constant',11,'Minutes/hour'),'1','Hours','hours')]


check('false-coreference-needs-semantic-review',false_coreference,accept=True)
try:json.loads('{"schema":2,"schema":2}',object_pairs_hook=unique_keys)
except ValueError:results.append(dict(name='duplicate-json-key',rejected=True))
else:raise AssertionError('duplicate-json-key')
result=validate(batch,base)
if len(sys.argv)>2:execute(result,Path(sys.argv[2]))
report=dict(total=len(results),passed=True,mechanical_semantic_limits_demonstrated=True,native_prefixes=len(result['native_requests']),native_executed='native_outputs' in result,results=results)
p=Path(sys.argv[1])
with p.open('x') as output:json.dump(report,output,indent=2);output.write('\n')
print(json.dumps({k:v for k,v in report.items() if k!='results'}))
