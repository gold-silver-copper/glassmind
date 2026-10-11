"""Authored protocol contracts; none of these fixtures is exposed to the teacher."""
import copy,json,re,sys
from pathlib import Path
from validate import validate


def source(kind,index):return dict(kind=kind,index=index)


def row(name,q,a,nodes,leaves,final):
    numbers=[dict(start=len(q[:m.start()].encode()),end=len(q[:m.end()].encode()),literal=m[0],value=m[0]) for m in re.finditer(r'[0-9]+',q)]
    annotations=[dict(answer_char_span=[m.start()+2,m.end()-2]) for m in re.finditer(r'<<.*?>>',a)]
    return dict(id=name,question=q,answer=a,numbers=numbers,annotations=annotations,expression_nodes=nodes,leaves=leaves,final_answer=final)


def leaf(value,choices,before=0):return dict(value=value,alternatives=[source(k,i) for k,i in choices],available_before=before)


def node(op,left,right,value,annotation=0):return dict(op=op,left=dict(leaf=left),right=dict(leaf=right),value=value,annotation=annotation)


def quote(row,field,start=0,end=None):
    raw=row[field].encode();end=len(raw) if end is None else end
    return dict(field=field,start=start,end=end,quote=raw[start:end].decode())


constants=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
x=row('correlation','Élodie has 7 red beads and 7 blue beads. How many beads?','Total: <<7+7=14>>14.\n#### 14',[node('add',0,1,'14')],[leaf('7',[('input',0),('input',1)]),leaf('7',[('input',0),('input',1)])],'14')
y=row('prior','Mira packed 7 green boxes and 8 red boxes. Each held 9 pencils. There were 15 crates nearby.','Boxes: <<7+8=15>>15. Pencils: <<15*9=135>>135.\n#### 135',[node('add',0,1,'15'),node('multiply',2,3,'135',1)],[leaf('7',[('input',0)]),leaf('8',[('input',1)]),leaf('15',[('input',3),('step',0)],1),leaf('9',[('input',2)],1)],'135')
z=row('constant','How many hours is 120 minutes?','Hours: <<120/60=2>>2.\n#### 2',[node('divide',0,1,'2')],[leaf('120',[('input',0)]),leaf('60',[('constant',11)])],'2')
batch=dict(constants=constants,rows=[x,y,z])


def annotation(row,refs):
    assignments=[dict(leaf=i,source=source(k,n),role='Authored fixture source role',evidence=[quote(row,'answer' if k=='step' else 'question')]) for i,(k,n) in enumerate(refs)]
    return dict(id=row['id'],status='resolved',reason='Authored mechanical protocol fixture',evidence=[quote(row,'question')],programs=[dict(assignments=assignments)])


base=dict(schema=1,rows=[annotation(x,[('input',0),('input',1)]),annotation(y,[('input',0),('input',1),('step',0),('input',2)]),annotation(z,[('input',0),('constant',11)])])
results=[]


def check(name,mutate=None,accept=False,change_batch=None):
    a,b=copy.deepcopy(base),copy.deepcopy(batch)
    if mutate:mutate(a)
    if change_batch:change_batch(b)
    try:r=validate(b,a)
    except (ValueError,TypeError,KeyError,IndexError) as error:
        assert not accept,(name,str(error));results.append(dict(name=name,rejected=True,reason=str(error)));return
    assert accept,name;assert r['semantic_approval'] is False
    results.append(dict(name=name,rejected=False,mechanically_accepted=r['mechanically_accepted']))


check('complete-native-shapes',accept=True)
check('extra-top-field',lambda a:a.update(hidden_gold=True))
check('missing-row',lambda a:a['rows'].pop())
check('duplicate-row',lambda a:a['rows'].__setitem__(1,copy.deepcopy(a['rows'][0])))
check('wrong-id',lambda a:a['rows'][0].update(id='unknown'))
check('unknown-status',lambda a:a['rows'][0].update(status='correct'))
check('refusal-with-program',lambda a:a['rows'][0].update(status='ambiguous'))
check('resolved-without-program',lambda a:a['rows'][0].update(programs=[]))
check('missing-leaf',lambda a:a['rows'][0]['programs'][0]['assignments'].pop())
check('duplicate-leaf',lambda a:a['rows'][0]['programs'][0]['assignments'][1].update(leaf=0))
check('boolean-leaf',lambda a:a['rows'][0]['programs'][0]['assignments'][0].update(leaf=False))
check('boolean-source',lambda a:a['rows'][0]['programs'][0]['assignments'][1]['source'].update(index=True))
check('negative-source',lambda a:a['rows'][0]['programs'][0]['assignments'][1]['source'].update(index=-1))
check('unknown-source-kind',lambda a:a['rows'][0]['programs'][0]['assignments'][1]['source'].update(kind='rationale'))
check('new-constant',lambda a:a['rows'][2]['programs'][0]['assignments'][1]['source'].update(index=20))
check('no-role',lambda a:a['rows'][0]['programs'][0]['assignments'][0].update(role=''))
check('bad-byte-range',lambda a:a['rows'][0]['evidence'][0].update(start=-1))
check('split-utf8-codepoint',lambda a:a['rows'][0]['evidence'][0].update(start=1))
check('invented-quote',lambda a:a['rows'][0]['evidence'][0].update(quote='not in source'))
check('unknown-evidence-field',lambda a:a['rows'][0]['evidence'][0].update(field='gold'))
check('extra-output-field',lambda a:a['rows'][0]['programs'][0].update(steps=[]))
check('input-cites-other-occurrence',lambda a:a['rows'][0]['programs'][0]['assignments'][0].update(evidence=[quote(x,'question',x['numbers'][1]['start'],x['numbers'][1]['end'])]))
check('constant-only-rationale',lambda a:a['rows'][2]['programs'][0]['assignments'][1].update(evidence=[quote(z,'answer')]))
check('step-cites-later-annotation',lambda a:a['rows'][1]['programs'][0]['assignments'][2].update(evidence=[quote(y,'answer',y['answer'].index('Pencils:'))]))
check('future-reference',lambda a:a['rows'][1]['programs'][0]['assignments'][2]['source'].update(index=1),change_batch=lambda b:b['rows'][1]['leaves'][2]['alternatives'].append(source('step',1)))


def correlate(a,bad=False,duplicate=False):
    row=a['rows'][0];row['status']='equivalent';other=copy.deepcopy(row['programs'][0])
    if not duplicate:
        other['assignments'][0]['source']['index']=1
        other['assignments'][1]['source']['index']=1 if bad else 0
    row['programs'].append(other)


check('correlated-commuting-pair',correlate,accept=True)
check('uncorrelated-repeated-input',lambda a:correlate(a,bad=True))
check('duplicate-program',lambda a:correlate(a,duplicate=True))
check('equivalent-needs-alternatives',lambda a:a['rows'][0].update(status='equivalent'))
check('noncommuting-numerical-coincidence',correlate,change_batch=lambda b:(b['rows'][0]['expression_nodes'][0].update(op='subtract',value='0'),b['rows'][0].update(final_answer='0')))
check('changed-annotated-arithmetic',change_batch=lambda b:b['rows'][0]['expression_nodes'][0].update(value='15'))
check('changed-final',change_batch=lambda b:b['rows'][0].update(final_answer='15'))
check('honest-nonacceptance',lambda a:a['rows'][0].update(status='label_error',programs=[]),accept=True)
# Mechanical checks alone deliberately cannot establish semantic source identity.
check('equal-valued-distractor-needs-parent-review',lambda a:a['rows'][1]['programs'][0]['assignments'][2].update(source=source('input',3),evidence=[quote(y,'question')]),accept=True)
report=dict(total=len(results),passed=True,semantic_limit_demonstrated=True,results=results)
p=Path(sys.argv[1]);p.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='results'}))
