"""Strict ranges and conjunction intersections from original raw statements."""
from pathlib import Path
import calendar,datetime,hashlib,itertools,json,re
base=Path(__file__).parent;work=base.parents[1]
def interval(s):
 try:
  if s['rank']=='deprecated' or set(s['qualifiers'])!={'P580','P582'}:return None
  ranges=[]
  for prop in ['P580','P582']:
   values=s['qualifiers'][prop]
   if len(values)!=1:return None
   sn=values[0]
   if sn['snaktype']!='value' or sn['property']!=prop or sn['datatype']!='time' or sn['datavalue']['type']!='time':return None
   v=sn['datavalue']['value'];p=v['precision']
   if p not in [9,10,11] or v['calendarmodel']!='http://www.wikidata.org/entity/Q1985727' or any(v[x]!=0 for x in ['timezone','before','after']):return None
   m=re.fullmatch(r'\+([0-9]{4})-([0-9]{2})-([0-9]{2})T00:00:00Z',v['time'])
   if not m:return None
   y,mo,d=map(int,m.groups())
   if not 1<=y<=9999 or mo>12 or d>31 or (mo==0 and d!=0):return None
   if mo and d:datetime.date(y,mo,d)
   if p==9:lo,hi=datetime.date(y,1,1),datetime.date(y,12,31)
   elif p==10:lo,hi=datetime.date(y,mo,1),datetime.date(y,mo,calendar.monthrange(y,mo)[1])
   else:lo=hi=datetime.date(y,mo,d)
   ranges.append((lo.isoformat(),hi.isoformat()))
  if ranges[0][0]>ranges[1][1]:return None
  return ranges
 except (KeyError,ValueError,TypeError):return None
def resolve(raw,index,date,constraints):
 matches=[]
 for sid,entity in sorted(index['entities'].items()):
  proof=[]
  for prop,obj in sorted(constraints):
   if obj not in entity['claims'].get(prop,[]):break
   supported=[]
   for c in raw['entities'].get(sid,{}).get('claims',{}).get(prop,[]):
    if c.get('mainsnak',{}).get('datavalue',{}).get('value',{}).get('id')!=obj:continue
    bounds=interval(c)
    if bounds and bounds[0][1]<date<bounds[1][0]:supported.append((c,bounds))
   if not supported:break
   c,bounds=min(supported,key=lambda x:x[0]['id'].lower());proof.append(dict(subject=sid,property=prop,object=obj,claim_id=c['id'],start_range=bounds[0],end_range=bounds[1]))
  else:matches.append(dict(subject=sid,witnesses=proof))
 return matches
cases=json.loads((base/'cases.json').read_text());authored=[]
for c in cases:
 if c['expected_status']!='matched':continue
 matches=resolve(c['raw'],c['index'],c['date'],c['constraints']);assert [m['subject'] for m in matches]==c['expected_subjects'],c['name'];authored.append(dict(name=c['name'],total=len(matches),matches=matches))
(base/'authored-oracle.json').write_text(json.dumps(authored,indent=2)+'\n')
source=work/'temporal-source';manifest=json.loads((source/'manifest.json').read_text());raw={'entities':{}}
for r in manifest['files']:
 b=(source/r['file']).read_bytes();assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'];v=json.loads(b)
 for sid,e in v['entities'].items():
  if sid in raw['entities']:assert raw['entities'][sid]==e
  raw['entities'][sid]=e
index=json.loads((work/'factmatch-source/index.json').read_text());pairs={};eligible=[]
for sid,e in sorted(index['entities'].items()):
 edges={}
 for prop in ['P108','P27']:
  for claim in raw['entities'].get(sid,{}).get('claims',{}).get(prop,[]):
   obj=claim.get('mainsnak',{}).get('datavalue',{}).get('value',{}).get('id');bounds=interval(claim)
   if obj not in e['claims'].get(prop,[]) or not bounds:continue
   edges.setdefault((prop,obj),[]).append(bounds);eligible.append(claim['id'])
 for a,b in itertools.combinations(sorted(edges),2):
  key=(a,b);dates=pairs.setdefault(key,set())
  for bounds in edges[a]+edges[b]:dates.update([bounds[0][1],bounds[1][0]])
  for first,second in itertools.product(edges[a],edges[b]):
   start=max(first[0][1],second[0][1]);end=min(first[1][0],second[1][0]);day=(datetime.date.fromisoformat(start)+datetime.timedelta(days=1)).isoformat()
   if day<end:dates.add(day)
real=[]
for constraints,dates in sorted(pairs.items()):
 question='Who '+' and '.join(('worked for ' if p=='P108' else 'was a citizen of ')+o for p,o in constraints)+'?'
 for date in sorted(dates):real.append(dict(question=question,date=date,constraints=constraints,expected_matches=resolve(raw,index,date,constraints)))
out=base/'real-contracts.json';assert not out.exists();out.write_text(json.dumps(real,indent=2)+'\n');summary=dict(authored_matched_contracts=len(authored),schema_eligible_claims=len(eligible),distinct_constraint_pairs=len(pairs),real_queries=len(real),nonempty_queries=sum(bool(r['expected_matches']) for r in real),matched_subjects=sorted({m['subject'] for r in real for m in r['expected_matches']}),real_sha256=hashlib.sha256(out.read_bytes()).hexdigest(),scope='Complete distinct eligible cached constraint pairs with boundaries and guaranteed interior dates; existing500validation-subjectseeded slice, not independent retrieval');(base/'oracle-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
