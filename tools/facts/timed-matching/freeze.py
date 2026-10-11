"""Public authored development fixtures and independent raw-statement oracle."""
import calendar, copy, datetime as dt, hashlib, json
from pathlib import Path
OUT=Path(__file__).resolve().parent
POLICY='latest_start_day < requested_day < earliest_end_day'
def digest(b): return hashlib.sha256(b).hexdigest()
def encode(x): return json.dumps(x,sort_keys=True,separators=(',',':')).encode()
def bounds(s):
    if s['rank']=='deprecated': return None
    q=s.get('qualifiers',{})
    if set(q)!= {'P580','P582'}: return None
    out=[]
    for p in ('P580','P582'):
        if not isinstance(q[p],list) or len(q[p])!=1: return None
        sn=q[p][0]
        if sn.get('snaktype')!='value' or sn.get('property')!=p or sn.get('datatype')!='time' or sn.get('datavalue',{}).get('type')!='time': return None
        r=sn['datavalue']['value']
        if set(r)!= {'time','timezone','before','after','precision','calendarmodel'}: return None
        if r['timezone'] or r['before'] or r['after'] or r['precision'] not in (9,10,11) or r['calendarmodel']!='http://www.wikidata.org/entity/Q1985727': return None
        import re
        m=re.fullmatch(r'\+(\d{4})-(\d{2})-(\d{2})T00:00:00Z',r['time'])
        if not m: return None
        y,mo,d=map(int,m.groups()); pr=r['precision']
        if not y or mo>12 or d>31 or (not mo and d) or (pr>=10 and not mo) or (pr==11 and not d): return None
        if mo and d>calendar.monthrange(y,mo)[1]: return None
        low=dt.date(y,1,1) if pr==9 else dt.date(y,mo,1) if pr==10 else dt.date(y,mo,d)
        high=dt.date(y,12,31) if pr==9 else dt.date(y,mo,calendar.monthrange(y,mo)[1]) if pr==10 else low
        out.append(dict(raw=r,earliest=str(low),latest=str(high)))
    return out if out[0]['earliest']<=out[1]['latest'] else None

def expected(index_bytes,records,date,constraints):
    cs=sorted(set(constraints)); matches=[]
    for subject in sorted(json.loads(index_bytes)['entities']):
        witnesses=[]
        for prop,obj in cs:
            for r in sorted(records,key=lambda r:r['statement']['id'].lower()):
                if (r['subject'],r['property'],r['object']) != (subject,prop,obj): continue
                b=bounds(r['statement'])
                if not b or not b[0]['latest']<date<b[1]['earliest']: continue
                edge=dict(subject=subject,property=prop,object=obj,subject_resolution='explicit_id',object_resolution='explicit_id',source_url=r['source_url'])
                witnesses.append(dict(edge=edge,claim_id=r['statement']['id'],rank=r['statement']['rank'],start=b[0],end=b[1],source_url=r['source_url'],raw_file=r['raw_file'],raw_sha256=r['raw_sha256'],index_sha256=digest(index_bytes),policy=POLICY)); break
            else: break
        if len(witnesses)==len(cs): matches.append(dict(subject=subject,witnesses=witnesses))
    return dict(date=date,constraints=[dict(property=p,object=o,object_resolution='explicit_id') for p,o in cs],matches=matches[:10],total_recorded_interval_matches=len(matches),returned=min(10,len(matches)),truncated=len(matches)>10)
def snak(p,t,precision=11):
    return dict(snaktype='value',property=p,datatype='time',datavalue=dict(type='time',value=dict(time=t,timezone=0,before=0,after=0,precision=precision,calendarmodel='http://www.wikidata.org/entity/Q1985727')))
def record(subject,prop,obj,n,start='1990-01-01',end='2010-01-01',sp=11,ep=11):
    s=dict(id=f'{subject}${n:08x}-0000-0000-0000-000000000000',type='statement',rank='normal',mainsnak=dict(snaktype='value',property=prop,datatype='wikibase-item',datavalue=dict(type='wikibase-entityid',value={'id':obj,'entity-type':'item'})),qualifiers={'P580':[snak('P580','+'+start+'T00:00:00Z',sp)],'P582':[snak('P582','+'+end+'T00:00:00Z',ep)]})
    return dict(subject=subject,property=prop,object=obj,source_url=f'https://www.wikidata.org/wiki/{subject}#{prop}',raw_file='0000000000000000.json',raw_sha256=digest(encode(s)),statement=s)
def question(cs): return 'Who '+' and '.join(('worked for ' if p=='P108' else 'was a citizen of ')+o for p,o in cs)+'?'
def freeze():
    cases=[]
    def add(name,rs,cs=None,date='2000-01-01',query=None,status='matched',extra=None):
        cs=cs or [('P108','Q2'),('P27','Q3')]
        entities={q:dict(aliases=[alias],claims={}) for q,alias in [('Q2','Employer'),('Q3','Country'),('Q4','Other')]}
        for r in rs:
            e=entities.setdefault(r['subject'],dict(aliases=[r['subject']],claims={}))
            e['claims'].setdefault(r['property'],[])
            if r['object'] not in e['claims'][r['property']]: e['claims'][r['property']].append(r['object'])
        if extra: extra(entities)
        ib=encode(dict(entities=entities)); asset=dict(version=1,index_sha256=digest(ib),records=rs)
        exp=expected(ib,rs,date,cs) if status=='matched' else {}
        cases.append(dict(name=name,index=ib.decode(),asset=asset,date=date,question=query or question(cs),status=status,expected=exp))
    base=[record('Q1','P108','Q2',1),record('Q1','P27','Q3',2,start='1999-01-01',end='2001-01-01')]
    add('distinct intervals',base)
    add('duplicate normalized',base,[('P108','Q2')]*2)
    add('same property distinct',base+[record('Q1','P108','Q4',3)],[('P108','Q2'),('P108','Q4')])
    add('three distinct',base+[record('Q1','P108','Q4',3)],[('P108','Q2'),('P27','Q3'),('P108','Q4')])
    add('missing interval',base[:1])
    rs=[]
    for n in range(100,112):
        rs += [record(f'Q{n}','P108','Q2',1,end='1999-01-01' if n<110 else '2010-01-01'),record(f'Q{n}','P27','Q3',2)]
    add('first ten static invalid',rs)
    add('full count twelve',[record(f'Q{n}',p,o,k) for n in range(100,112) for k,(p,o) in enumerate([('P108','Q2'),('P27','Q3')],1)])
    add('distinct claims choose eligible', [record('Q1','P108','Q2',0,end='1999-01-01')]+base+[record('Q1','P108','Q2',5)])
    for date in ['1990-01-01','1990-01-02','2009-12-31','2010-01-01']:
        add('strict boundary '+date,[base[0]], [('P108','Q2')]*2,date)
    for precision,start,end,dates in [(10,'2000-02-00','2000-04-00',['2000-02-28','2000-02-29','2000-03-01','2000-03-31','2000-04-01']),(9,'1999-00-00','2001-00-00',['1999-12-31','2000-01-01','2000-12-31','2001-01-01'])]:
        for date in dates: add('precision '+str(precision)+' '+date,[record('Q1','P108','Q2',1,start,end,precision,precision)],[('P108','Q2')]*2,date)
    mutations={
      'missing start':lambda s:s['qualifiers'].pop('P580'),
      'open end':lambda s:s['qualifiers'].pop('P582'),
      'extra qualifier':lambda s:s['qualifiers'].update(P585=[]),
      'duplicate start':lambda s:s['qualifiers']['P580'].append(copy.deepcopy(s['qualifiers']['P580'][0])),
      'nonvalue end':lambda s:s['qualifiers']['P582'][0].update(snaktype='somevalue'),
      'deprecated':lambda s:s.update(rank='deprecated'),
      'timezone':lambda s:s['qualifiers']['P580'][0]['datavalue']['value'].update(timezone=1),
      'uncertainty':lambda s:s['qualifiers']['P580'][0]['datavalue']['value'].update(before=1),
      'precision unsupported':lambda s:s['qualifiers']['P580'][0]['datavalue']['value'].update(precision=8),
      'non Gregorian':lambda s:s['qualifiers']['P580'][0]['datavalue']['value'].update(calendarmodel='other'),
    }
    for name,mut in mutations.items():
        rs=copy.deepcopy(base);mut(rs[0]['statement']);add(name,rs)
    rs=copy.deepcopy(base);rs[0]['statement']['qualifiers'].pop('P582');r=record('Q1','P108','Q2',3);r['statement']['qualifiers'].pop('P580');add('split bounds cannot stitch',rs+[r])
    add('alias collision only one matching',base,query='Who worked for Shared and was a citizen of Q3?',status='unknown',extra=lambda es:[es[q]['aliases'].append('Shared') for q in ['Q2','Q4']])
    for q in ['Who worked for Missing and was a citizen of Q3?','Who worked for Q999 and was a citizen of Q3?']:
        add(q,base,query=q,status='unknown' if 'Missing' in q else 'matched')
        if 'Q999' in q: cases[-1]['expected']=expected(cases[-1]['index'].encode(),base,'2000-01-01',[('P108','Q999'),('P27','Q3')])
    for q in ['Who was born in Q2 and worked for Q2?','Who died in Missing and worked for Q2?','Who worked for Q2 and was a citizen of Q3? trailing','Who did not work for Q2 and was a citizen of Q3?','Who worked for Q2\nand was a citizen of Q3?','Who worked for Q2 and was a citizen of Q3?\0','Who worked for Q2?','Who worked for q2 and was a citizen of Q3?','Who worked for Q2 and was a citizen of Missing? trailing']:
        add(q,base,query=q,status='refused')
    for date in ['2000-2-01','1900-02-29','0000-01-01','2001-02-29','2000-04-31','2000-01-01T00:00:00Z']:
        add('invalid '+date,base,date=date,status='refused')
    add('leap exact valid',base,date='2000-02-29')
    (OUT/'contracts.json').write_bytes(encode(cases)+b'\n')
    (OUT/'freeze.sha256').write_text('\n'.join(f'{digest((OUT/p).read_bytes())}  {p}' for p in ['PLAN.md','freeze.py','contracts.json'])+'\n')
    print(len(cases))
if __name__=='__main__': freeze()
