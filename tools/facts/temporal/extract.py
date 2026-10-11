"""Offline lossless P108/P27 statement extraction from hash-verified supplied bytes."""
import argparse,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--index',type=Path,required=True);p.add_argument('--index-manifest',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
def read(path,cap):
 with path.open('rb') as f:b=f.read(cap+1)
 if len(b)>cap:raise ValueError('byte cap: '+str(path))
 return b
def sha(b):return hashlib.sha256(b).hexdigest()
def unique_pairs(pairs):
 d={}
 for k,v in pairs:
  if k in d:raise ValueError('duplicate JSON key: '+k)
  d[k]=v
 return d
def parse(b):return json.loads(b,object_pairs_hook=unique_pairs)
manifest=parse(read(a.source/'manifest.json',1048576));im=parse(read(a.index_manifest,1048576))
ib=read(a.index,32*1048576);assert sha(ib)==manifest['index_sha256']==im['index.json']['sha256'];assert len(ib)==im['index.json']['bytes']
index=parse(ib)['entities'];records=[];skipped=0;total=0;seen=set()
assert len(manifest['files'])<=128
for f in sorted(manifest['files'],key=lambda f:f['file']):
 assert re.fullmatch(r'[a-f0-9]{16}\.json',f['file']) and f['file'] not in seen;seen.add(f['file'])
 raw=read(a.source/f['file'],8*1048576);total+=len(raw);assert total<=64*1048576
 assert sha(raw)==f['sha256'] and len(raw)==f['bytes']
 for sid,e in sorted(parse(raw)['entities'].items()):
  if sid not in index:continue
  for prop in ['P108','P27']:
   for c in e.get('claims',{}).get(prop,[]):
    obj=c.get('mainsnak',{}).get('datavalue',{}).get('value',{}).get('id')
    if obj not in index[sid]['claims'].get(prop,[]):skipped+=1;continue
    assert len(json.dumps(c).encode())<=65536
    records.append(dict(subject=sid,property=prop,object=obj,source_url=f'https://www.wikidata.org/wiki/{sid}#{prop}',raw_file=f['file'],raw_sha256=f['sha256'],statement=c))
    assert len(records)<=100000
records.sort(key=lambda r:(r['statement']['id'],r['raw_file']))
asset=dict(version=1,index_sha256=sha(ib),records=records)
b=(json.dumps(asset,sort_keys=True,separators=(',',':'))+'\n').encode();assert len(b)<=16*1048576
a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(b)
print(json.dumps(dict(records=len(records),bytes=len(b),sha256=sha(b),raw_bytes=total,skipped_nonindexed=skipped)))
