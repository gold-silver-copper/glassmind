"""Authored CLI/library equality and independent datetime reference over original bytes."""
import argparse,calendar,datetime,hashlib,json,subprocess,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);p.add_argument('--source',type=Path,required=True);p.add_argument('--data',type=Path,required=True);a=p.parse_args()
here=Path(__file__).resolve().parent;binary=a.run/'target/debug/glassmind-facts';library=a.run/'target/debug/examples/temporal_proof';out=a.run/'reports';out.mkdir(exist_ok=True);scratch=a.run/'tmp/proof';scratch.mkdir(exist_ok=True)
def sha(b):return hashlib.sha256(b).hexdigest()
def call(index,asset,queries):
 raw=subprocess.run([str(library),str(index),str(asset)],input=''.join(json.dumps([d,q])+'\n' for d,q in queries).encode(),capture_output=True,check=True,timeout=60).stdout
 lib=[json.loads(x) for x in raw.splitlines()];assert len(lib)==len(queries)
 for (d,q),v in zip(queries,lib):
  cli=json.loads(subprocess.run([str(binary),'verify-at','--index',str(index),'--temporal',str(asset),'--date',d,q],capture_output=True,check=True,timeout=30).stdout)
  assert cli==v,('CLI/library entire JSON mismatch',d,q,cli,v)
 return lib
def bounds(s):
 if s.get('rank')=='deprecated':return None
 q=s.get('qualifiers',{})
 if set(q)!={'P580','P582'}:return None
 result=[]
 for prop in ['P580','P582']:
  if len(q[prop])!=1:return None
  snak=q[prop][0]
  if snak.get('snaktype')!='value' or snak.get('datatype')!='time':return None
  r=snak['datavalue']['value'];prec=r['precision'];t=r['time']
  if prec not in [9,10,11] or r['calendarmodel']!='http://www.wikidata.org/entity/Q1985727' or any(r[x]!=0 for x in ['timezone','before','after']):return None
  if not re.fullmatch(r'\+[0-9]{4}-[0-9]{2}-[0-9]{2}T00:00:00Z',t):return None
  y,m,d=map(int,[t[1:5],t[6:8],t[9:11]])
  if not 1<=y<=9999 or m>12 or d>31 or (m==0 and d!=0):return None
  if m and d:
   try:datetime.date(y,m,d)
   except ValueError:return None
  if prec==9:lo=datetime.date(y,1,1);hi=datetime.date(y,12,31)
  elif prec==10:
   if not m:return None
   lo=datetime.date(y,m,1);hi=datetime.date(y,m,calendar.monthrange(y,m)[1])
  else:
   try:lo=hi=datetime.date(y,m,d)
   except ValueError:return None
  result.append(dict(raw=r,earliest=lo.isoformat(),latest=hi.isoformat()))
 if result[0]['earliest']>result[1]['latest']:return None
 return result
def expected(record,date,index_hash,subject_resolution='explicit_id',object_resolution='explicit_id'):
 s=record['statement'];b=bounds(s);assert b and b[0]['latest']<date<b[1]['earliest']
 edge={k:record[k] for k in ['subject','property','object','source_url']};edge.update(subject_resolution=subject_resolution,object_resolution=object_resolution)
 return dict(status='supported',date=date,scope='one_recorded_bounded_interval_calendar_day',witness=dict(edge=edge,claim_id=s['id'],rank=s['rank'],start=b[0],end=b[1],source_url=record['source_url'],raw_file=record['raw_file'],raw_sha256=record['raw_sha256'],index_sha256=index_hash,policy='latest_start_day < requested_day < earliest_end_day'))
index=here/'index.json';ih=sha(index.read_bytes());authored=[]
for i,c in enumerate(json.loads((here/'contracts.json').read_text())):
 records=[]
 for s in c['statements']:
  prop=s['mainsnak']['property'];obj=s['mainsnak']['datavalue']['value']['id']
  records.append(dict(subject='Q1',property=prop,object=obj,source_url=f'https://www.wikidata.org/wiki/Q1#{prop}',raw_file='0000000000000000.json',raw_sha256='0'*64,statement=s))
 path=scratch/f'case-{i:03}.json';path.write_text(json.dumps(dict(version=1,index_sha256=ih,records=records)))
 result=call(index,path,[(c['date'],c['question'])])[0];assert result['status']==c['status'],(c,result)
 if c['status']=='supported':
  want=expected(records[0],c['date'],ih,'explicit_id' if ' Q1 ' in c['question'] else 'alias','explicit_id' if re.search(r'Q[23]\?',c['question']) else 'alias')
  assert result==want,(c,result,want)
 authored.append(dict(name=c['name'],actual=result))
(out/'authored-proof.json').write_text(json.dumps(authored,indent=2)+'\n')
# Assemble expected real records directly from ORIGINAL response bytes, not extractor output.
manifest=json.loads((a.source/'manifest.json').read_text());idx=a.data/'index.json';ih=sha(idx.read_bytes());assert ih==manifest['index_sha256'];store=json.loads(idx.read_bytes())['entities'];original=[]
for f in manifest['files']:
 raw=(a.source/f['file']).read_bytes();assert sha(raw)==f['sha256'] and len(raw)==f['bytes']
 for sid,e in json.loads(raw)['entities'].items():
  if sid not in store:continue
  for prop in ['P108','P27']:
   for s in e.get('claims',{}).get(prop,[]):
    obj=s.get('mainsnak',{}).get('datavalue',{}).get('value',{}).get('id')
    if obj not in store[sid]['claims'].get(prop,[]):continue
    original.append(dict(subject=sid,property=prop,object=obj,source_url=f'https://www.wikidata.org/wiki/{sid}#{prop}',raw_file=f['file'],raw_sha256=f['sha256'],statement=s))
asset=json.loads((a.run/'temporal.json').read_text());assert sorted(original,key=lambda r:r['statement']['id'])==sorted(asset['records'],key=lambda r:r['statement']['id']), 'lossless original-statement equality'
qualified=[r for r in original if bounds(r['statement'])];assert len(qualified)==20,len(qualified)
real=[];subjects=set()
for r in sorted(qualified,key=lambda r:r['statement']['id'].lower()):
 if r['subject'] in subjects:continue
 b=bounds(r['statement']);start=datetime.date.fromisoformat(b[0]['latest']);end=datetime.date.fromisoformat(b[1]['earliest'])
 if (end-start).days<2:continue
 interior=(start+(end-start)//2).isoformat();dates=[interior,start.isoformat(),end.isoformat(),(datetime.date.fromisoformat(b[0]['earliest'])-datetime.timedelta(days=1)).isoformat(),(datetime.date.fromisoformat(b[1]['latest'])+datetime.timedelta(days=1)).isoformat()]
 peers=[p for p in qualified if all(p[k]==r[k] for k in ['subject','property','object'])]
 if any(any(bounds(p['statement'])[0]['latest']<d<bounds(p['statement'])[1]['earliest'] for p in peers) for d in dates[1:]):continue
 question=f"Did {r['subject']} work for {r['object']}?" if r['property']=='P108' else f"Was {r['subject']} a citizen of {r['object']}?"
 results=call(idx,a.run/'temporal.json',[(d,question) for d in dates]);assert [v['status'] for v in results]==['supported','unknown','unknown','unknown','unknown']
 winner=min((p for p in peers if bounds(p['statement'])[0]['latest']<interior<bounds(p['statement'])[1]['earliest']),key=lambda p:p['statement']['id'].lower())
 assert results[0]==expected(winner,interior,ih)
 real.append(dict(question=question,dates=dates,results=results));subjects.add(r['subject'])
 if len(subjects)==3:break
assert len(subjects)==3
(out/'real-proof.json').write_text(json.dumps(real,indent=2)+'\n')
# Full affected cached dev: every eligible statement, same immutable rules.
full=[]
for r in sorted(qualified,key=lambda r:r['statement']['id'].lower()):
 b=bounds(r['statement']);start=datetime.date.fromisoformat(b[0]['latest']);end=datetime.date.fromisoformat(b[1]['earliest'])
 dates=[(start+(end-start)//2).isoformat(),start.isoformat(),end.isoformat(),(start+datetime.timedelta(days=1)).isoformat(),(end-datetime.timedelta(days=1)).isoformat()]
 peers=sorted([p for p in qualified if all(p[k]==r[k] for k in ['subject','property','object'])],key=lambda p:p['statement']['id'].lower())
 question=f"Did {r['subject']} work for {r['object']}?" if r['property']=='P108' else f"Was {r['subject']} a citizen of {r['object']}?"
 results=call(idx,a.run/'temporal.json',[(d,question) for d in dates])
 for d,result in zip(dates,results):
  winners=[p for p in peers if bounds(p['statement'])[0]['latest']<d<bounds(p['statement'])[1]['earliest']]
  if winners:assert result==expected(winners[0],d,ih)
  else:assert result['status']=='unknown'
 full.append(dict(claim_id=r['statement']['id'],question=question,dates=dates,results=results))
(out/'full-cache-proof.json').write_text(json.dumps(full,indent=2)+'\n')
summary=dict(authored=len(authored),authored_full_supported_witnesses=sum(x['actual']['status']=='supported' for x in authored),full_cache_queries=sum(len(x["results"]) for x in full),real_subjects=len(subjects),real_queries=sum(len(x['results']) for x in real),lossless_records=len(original),qualifying_statements=len(qualified),cli_library_full_json_equal=True)
(out/'proof-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
