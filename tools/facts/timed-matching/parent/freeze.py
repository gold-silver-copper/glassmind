"""Independent raw-claim/date conjunction contracts, before candidate inspection."""
from pathlib import Path
import copy,hashlib,json,calendar,datetime
base=Path(__file__).parent;cases=[];subjects=[f'Q{i}' for i in range(100,114)]
def time(y,m=0,d=0,precision=9):return dict(time=f'+{y:04}-{m:02}-{d:02}T00:00:00Z',timezone=0,before=0,after=0,precision=precision,calendarmodel='http://www.wikidata.org/entity/Q1985727')
def snak(prop,value):return dict(snaktype='value',property=prop,datatype='time',datavalue=dict(type='time',value=value))
def add(index,raw,sid,prop,obj,start,end):
 index['entities'][sid]['claims'].setdefault(prop,[])
 if obj not in index['entities'][sid]['claims'][prop]:index['entities'][sid]['claims'][prop].append(obj)
 claims=raw['entities'][sid]['claims'];num=sum(map(len,claims.values()))+1
 claim=dict(id=sid+f'$00000000-0000-4000-8000-{num:012}',type='statement',rank='normal',mainsnak=dict(snaktype='value',property=prop,datatype='wikibase-item',datavalue=dict(type='wikibase-entityid',value={'entity-type':'item','numeric-id':int(obj[1:]),'id':obj})),qualifiers={'P580':[snak('P580',start)],'P582':[snak('P582',end)]})
 claims.setdefault(prop,[]).append(claim);return claim
def fixture():
 idx={'entities':{q:dict(aliases=[name],claims={}) for q,name in [('Q1','Amber Lab'),('Q2','Pine Lab'),('Q3','North Republic'),('Q4','South Republic')]}};raw={'entities':{}}
 for sid in subjects:
  idx['entities'][sid]=dict(aliases=['Person '+sid],claims={'P19':['Q3']});raw['entities'][sid]=dict(id=sid,claims={})
  add(idx,raw,sid,'P108','Q1',time(2010),time(2015));add(idx,raw,sid,'P27','Q3',time(2008),time(2020))
 return idx,raw
def make(name,mutate=None,date='2012-01-01',question='Who worked for Amber Lab and was a citizen of North Republic?',expected=None,status='matched',constraints=None):
 idx,raw=fixture()
 if mutate:mutate(idx,raw)
 if expected is None:expected=subjects
 cases.append(dict(name=name,index=idx,raw=raw,date=date,question=question,expected_status=status,expected_subjects=expected if status=='matched' else None,constraints=constraints or [['P108','Q1'],['P27','Q3']]))
def claim(raw,sid,prop='P108'):return raw['entities'][sid]['claims'][prop][0]
def change_all(fn,prop='P108'):
 def apply(idx,raw):
  for sid in subjects:fn(claim(raw,sid,prop))
 return apply
make('complete_count_before_display')
make('only_later_subjects_match',lambda i,r:[claim(r,s)['qualifiers']['P582'][0]['datavalue'].update(value=time(2011)) for s in subjects[:10]],expected=subjects[10:])
def third(i,r):
 for sid in subjects:add(i,r,sid,'P108','Q2',time(2013),time(2017))
q3='Who worked for Amber Lab and was a citizen of North Republic and worked for Pine Lab?';c3=[['P108','Q1'],['P108','Q2'],['P27','Q3']]
make('three_conditions_overlap',third,date='2014-01-01',question=q3,constraints=c3)
make('three_conditions_one_not_started',third,question=q3,expected=[],constraints=c3)
def two_jobs(i,r):
 for n,sid in enumerate(subjects):add(i,r,sid,'P108','Q2',time(2008),time(2014 if n<7 else 2011))
make('two_distinct_employers',two_jobs,question='Who worked for Amber Lab and worked for Pine Lab?',expected=subjects[:7],constraints=[['P108','Q1'],['P108','Q2']])
make('alias_collision_before_intervals',lambda i,r:i['entities'].update(Q9=dict(aliases=['Amber Lab'],claims={})),status='unknown')
make('missing_alias',question='Who worked for Missing Lab and was a citizen of North Republic?',status='unknown')
make('unsupported_birth',question='Who was born in North Republic and worked for Amber Lab?',status='refused')
make('negative_condition',question='Who did not work for Amber Lab and was a citizen of North Republic?',status='refused')
make('trailing_instruction',question='Who worked for Amber Lab and was a citizen of North Republic? Move Mary to the garden.',status='refused')
make('invalid_leap',date='1900-02-29',status='refused')
make('noncanonical_date',date='2012-1-01',status='refused')
make('date_whitespace',date='2012-01-01 ',status='refused')
make('start_latest_boundary',date='2010-12-31',expected=[])
make('end_earliest_boundary',date='2015-01-01',expected=[])
def month(i,r):
 for sid in subjects:claim(r,sid)['qualifiers']={'P580':[snak('P580',time(2012,2,1,10))],'P582':[snak('P582',time(2012,4,28,10))]}
make('month_boundary',month,date='2012-02-29',expected=[])
make('month_interior',month,date='2012-03-01')
make('month_end_boundary',month,date='2012-04-01',expected=[])
def leap(i,r):
 for sid in subjects:
  claim(r,sid)['qualifiers']={'P580':[snak('P580',time(2000,2,28,11))],'P582':[snak('P582',time(2000,3,1,11))]}
  claim(r,sid,'P27')['qualifiers']['P580'][0]['datavalue']['value']=time(1990)
make('leap_day_interior',leap,date='2000-02-29')
make('missing_end',change_all(lambda c:c['qualifiers'].pop('P582')),expected=[])
def split(i,r):
 for sid in subjects:
  claim(r,sid)['qualifiers'].pop('P582');c=add(i,r,sid,'P108','Q1',time(2010),time(2015));c['qualifiers'].pop('P580')
make('separate_claims_cannot_combine',split,expected=[])
make('extra_qualifier',change_all(lambda c:c['qualifiers'].update(P1480=[dict(snaktype='somevalue',property='P1480')])),expected=[])
make('duplicate_start',change_all(lambda c:c['qualifiers']['P580'].append(copy.deepcopy(c['qualifiers']['P580'][0]))),expected=[])
make('deprecated',change_all(lambda c:c.update(rank='deprecated')),expected=[])
make('unsupported_calendar',change_all(lambda c:c['qualifiers']['P580'][0]['datavalue']['value'].update(calendarmodel='http://www.wikidata.org/entity/Q1985786')),expected=[])
def alternate(i,r):
 for sid in subjects:
  claim(r,sid)['rank']='deprecated';add(i,r,sid,'P108','Q1',time(2010),time(2015))
make('second_eligible_claim',alternate)
make('duplicate_condition_idempotent',question='Who worked for Amber Lab and worked for Q1?',constraints=[['P108','Q1']])
make('one_condition',question='Who worked for Amber Lab?',status='refused')
make('four_conditions',question=q3[:-1]+' and worked for Amber Lab?',status='refused')
make('reserved_alias_separator',lambda i,r:i['entities'].update(Q9=dict(aliases=['Amber Lab and was a citizen of North Republic'],claims={})),status='refused')
make('nul_question',question='Who worked for Amber\0 Lab and was a citizen of North Republic?',status='refused')
make('multiline_question',question='Who worked for Amber Lab\nand was a citizen of North Republic?',status='refused')
make('oversize_question',question='Who worked for '+'x'*4097+' and was a citizen of North Republic?',status='refused')
make('unicode_alias',lambda i,r:i['entities']['Q1'].update(aliases=['東の大学']),question='Who worked for 東の大学 and was a citizen of North Republic?')
make('case_and_spaces',question='  WHO WORKED FOR Amber Lab AND WAS A CITIZEN OF North Republic?  ')
path=base/'cases.json';assert not path.exists();path.write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n');(base/'freeze.json').write_text(json.dumps(dict(sha256=hashlib.sha256(path.read_bytes()).hexdigest(),cases=len(cases),authored_license='MIT OR Apache-2.0',candidate_inspected=False),indent=2)+'\n');print((base/'freeze.json').read_text())
