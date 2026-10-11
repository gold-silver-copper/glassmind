"""Authored development inputs/expectations; never imports the extractor."""
import copy,json,hashlib
from pathlib import Path
out=Path(__file__).parent
store={'entities':{'Q1':{'aliases':['Alice','Shared'],'claims':{'P108':['Q2'],'P27':['Q3']}},'Q4':{'aliases':['Shared'],'claims':{}},'Q2':{'aliases':['Company'],'claims':{}}}}
def bound(p,time,precision=9):
 return {'snaktype':'value','property':p,'datatype':'time','datavalue':{'type':'time','value':{'time':time,'timezone':0,'before':0,'after':0,'precision':precision,'calendarmodel':'http://www.wikidata.org/entity/Q1985727'}}}
statement={'id':'Q1$11111111-1111-1111-1111-111111111111','rank':'normal','type':'statement','mainsnak':{'snaktype':'value','property':'P108','datatype':'wikibase-item','datavalue':{'type':'wikibase-entityid','value':{'entity-type':'item','numeric-id':2,'id':'Q2'}}},'qualifiers':{'P580':[bound('P580','+1960-00-00T00:00:00Z')],'P582':[bound('P582','+1989-00-00T00:00:00Z')]}}
cases=[]
def add(name,date='1980-06-01',status='supported',question='Did Alice work for Company?',edit=None):
 s=copy.deepcopy(statement)
 if edit:edit(s)
 cases.append(dict(name=name,date=date,question=question,status=status,statements=[s]))
for name,date,status in [('year interior','1980-06-01','supported'),('year start','1960-12-31','unknown'),('year end','1989-01-01','unknown'),('year outside','1990-01-01','unknown'),('year first','1961-01-01','supported'),('year last','1988-12-31','supported'),('leap2000','2000-02-29','unknown'),('leap1900','1900-02-29','refused'),('leap2100','2100-02-29','refused'),('yearzero','0000-01-01','refused'),('nonascii','２０２０-01-01','refused'),('padding',' 1980-06-01','refused'),('trailing','1980-06-01\n','refused'),('short','1980-6-01','refused'),('invalidmonth','1980-13-01','refused'),('invalidday','1980-04-31','refused')]:add(name,date,status)
def times(s,a,b,p):
 s['qualifiers']['P580']=[bound('P580',a,p)];s['qualifiers']['P582']=[bound('P582',b,p)]
for name,date,status in [('month interior','2000-02-29','supported'),('month start','2000-01-31','unknown'),('month end','2000-03-01','unknown')]:add(name,date,status,edit=lambda s:times(s,'+2000-01-00T00:00:00Z','+2000-03-00T00:00:00Z',10))
for name,date,status in [('day interior','2000-02-29','supported'),('day start','2000-02-28','unknown'),('day end','2000-03-01','unknown')]:add(name,date,status,edit=lambda s:times(s,'+2000-02-28T00:00:00Z','+2000-03-01T00:00:00Z',11))
add('filled year same uncertainty',edit=lambda s:times(s,'+1960-03-12T00:00:00Z','+1989-09-14T00:00:00Z',9))
add('filled year boundary','1960-11-30','unknown',edit=lambda s:times(s,'+1960-03-12T00:00:00Z','+1989-09-14T00:00:00Z',9))
for key,value in [('time','-1960-00-00T00:00:00Z'),('time','+0000-00-00T00:00:00Z'),('time','+1960-00-12T00:00:00Z'),('time','+1960-13-00T00:00:00Z'),('time','+1900-02-29T00:00:00Z'),('time','+1960-01-01T01:00:00Z'),('precision',8),('precision',12),('timezone',1),('before',1),('after',1),('calendarmodel','http://www.wikidata.org/entity/Q1985786')]:
 add(f'bad bound {key} {value}',status='unknown',edit=lambda s,k=key,v=value:s['qualifiers']['P580'][0]['datavalue']['value'].__setitem__(k,v))
add('open start',status='unknown',edit=lambda s:s['qualifiers'].pop('P580'))
add('open end',status='unknown',edit=lambda s:s['qualifiers'].pop('P582'))
add('extra qualifier',status='unknown',edit=lambda s:s['qualifiers'].__setitem__('P1480',[]))
add('duplicate start',status='unknown',edit=lambda s:s['qualifiers']['P580'].append(copy.deepcopy(s['qualifiers']['P580'][0])))
for kind in ['somevalue','novalue']:add(kind,status='unknown',edit=lambda s,k=kind:s['qualifiers']['P580'][0].__setitem__('snaktype',k))
add('deprecated',status='unknown',edit=lambda s:s.__setitem__('rank','deprecated'))
add('inverted',status='unknown',edit=lambda s:times(s,'+1990-00-00T00:00:00Z','+1989-00-00T00:00:00Z',9))
add('overlap',status='unknown',edit=lambda s:times(s,'+1980-00-00T00:00:00Z','+1980-00-00T00:00:00Z',9))
for name,q,status in [('ambiguous alias','Did Shared work for Company?','unknown'),('missing alias','Did Nobody work for Company?','unknown'),('explicit IDs','Did Q1 work for Q2?','supported'),('missing edge','Did Q1 work for Q3?','unknown'),('negative','Did Alice not work for Company?','refused'),('bad grammar','Alice worked for Company','refused'),('wrong property','Was Alice born in Company?','refused'),('endpoint cap','Did '+'x'*257+' work for Company?','refused'),('question cap','x'*4097,'refused')]:add(name,status=status,question=q)
add('two open claims cannot combine',status='unknown',edit=lambda s:s['qualifiers'].pop('P580'))
s=copy.deepcopy(statement);s['id']='Q1$22222222-2222-2222-2222-222222222222';s['qualifiers'].pop('P582');cases[-1]['statements'].append(s)
add('citizenship edge only object',question='Was Q1 a citizen of Q3?',edit=lambda s:(s['mainsnak'].__setitem__('property','P27'),s['mainsnak']['datavalue']['value'].update({'id':'Q3','numeric-id':3})))
for name,obj in [('index.json',store),('contracts.json',cases)]:
 (out/name).write_text(json.dumps(obj,indent=2)+'\n')
(out/'frozen.sha256').write_text(''.join(hashlib.sha256((out/n).read_bytes()).hexdigest()+'  '+n+'\n' for n in ['PLAN.md','freeze_contracts.py','index.json','contracts.json']))
print(len(cases),'contracts frozen')
