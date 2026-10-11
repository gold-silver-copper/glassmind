"""Independent witness and malformed-asset checks; no expected labels changed."""
from pathlib import Path
import calendar,copy,hashlib,json,subprocess
base=Path(__file__).parent;work=base.parents[1];frozen=work/'temporal-run/frozen-candidate';out=base/'review';out.mkdir()
def bounds(s):
 v=s['datavalue']['value'];y,m,d=map(int,v['time'][1:11].split('-'));p=v['precision']
 lo=(y,1,1) if p==9 else (y,m,1) if p==10 else (y,m,d)
 hi=(y,12,31) if p==9 else (y,m,calendar.monthrange(y,m)[1]) if p==10 else (y,m,d)
 return [f'{y:04}-{m:02}-{d:02}' for y,m,d in [lo,hi]]
def verify_witness(value,asset,index,date):
 w=value['witness'];edge=w['edge'];rec=next(r for r in asset['records'] if r['statement']['id']==w['claim_id'])
 assert [edge[k] for k in ['subject','property','object']]==[rec[k] for k in ['subject','property','object']]
 assert w['rank']==rec['statement']['rank']
 for k in ['source_url','raw_file','raw_sha256']:assert w[k]==rec[k]
 assert w['index_sha256']==hashlib.sha256(index).hexdigest()
 for label,prop in [('start','P580'),('end','P582')]:
  snak=rec['statement']['qualifiers'][prop][0];assert w[label]['raw']==snak['datavalue']['value'];assert [w[label]['earliest'],w[label]['latest']]==bounds(snak)
 assert value['date']==date and w['start']['latest']<date<w['end']['earliest']
 assert value['scope']=='one_recorded_bounded_interval_calendar_day'
 assert w['policy']=='latest_start_day < requested_day < earliest_end_day'
 return rec
authored=[]
for c in json.loads((base/'cases.json').read_text()):
 d=base/'candidate'/c['name'];value=json.loads((d/'library.stdout').read_text());asset=json.loads((d/'temporal.json').read_text())
 if value['status']=='supported':
  rec=verify_witness(value,asset,(d/'index.json').read_bytes(),c['date']);raw=json.loads((base/'inputs'/c['name']/'entity.json').read_text())
  assert rec['statement'] in raw['entities'][rec['subject']]['claims'][rec['property']]
  authored.append(c['name'])
realpath=work/'coordinator/temporal-facts-audit/real-contracts.json'
assert hashlib.sha256(realpath.read_bytes()).hexdigest()=='49befcfaf51dc867b4a0ccb58938815479143b7cece14275ebf3664d0eec2358'
index=work/'factmatch-source/index.json';assetpath=frozen/'temporal.json';asset=json.loads(assetpath.read_text());real=[]
for c in json.loads(realpath.read_text())['contracts']:
 for q in c['queries']:
  p=subprocess.run([str(frozen/'binary/temporal_proof'),str(index),str(assetpath)],input=json.dumps([q['date'],c['question']])+'\n',capture_output=True,text=True,timeout=30);assert p.returncode==0,p.stderr
  value=json.loads(p.stdout);assert value['status']==q['status'],(c,q,value)
  if value['status']=='supported':
   rec=verify_witness(value,asset,index.read_bytes(),q['date']);assert rec['statement']['id']==c['claim_id'];assert rec['raw_sha256']==c['raw_sha256'];assert rec['raw_file']==c['raw_file']
  real.append(dict(subject=c['subject'],date=q['date'],result=value))
neg=[];seed=json.loads((base/'candidate/year_interior/temporal.json').read_text());seedindex=base/'candidate/year_interior/index.json'
def reject(name,data):
 pth=out/(name+'.json');pth.write_bytes(data)
 p=subprocess.run([str(frozen/'binary/temporal_proof'),str(seedindex),str(pth)],input='',capture_output=True,text=True,timeout=30)
 assert p.returncode!=0 and not p.stdout,(name,p.stdout,p.stderr)
 neg.append(dict(name=name,exit=p.returncode,error=p.stderr.strip()))
for name in ['index_hash','edge_object','precomputed_bounds','version','record_limit']:
 v=copy.deepcopy(seed)
 if name=='index_hash':v['index_sha256']='0'*64
 if name=='edge_object':v['records'][0]['object']='Q3'
 if name=='precomputed_bounds':v['records'][0]['start']={'earliest':'2000-01-01','latest':'2000-01-01'}
 if name=='version':v['version']=2
 if name=='record_limit':v['records']=[None]*100001
 reject(name,json.dumps(v).encode())
reject('byte_limit',b' '*(16*1024*1024+1))
reject('duplicate_qualifiers',json.dumps(seed).replace('"qualifiers":','"qualifiers":{},"qualifiers":',1).encode())
report=dict(authored_supported_witnesses=authored,real_queries=real,loader_negatives=neg)
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(authored_witnesses=len(authored),real_queries=len(real),loader_negatives=len(neg))))
