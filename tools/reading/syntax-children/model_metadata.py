"""Inspect bounded model metadata, verify identical learned tagger tables."""
import hashlib,json,pathlib,struct,sys
baseline,out=map(pathlib.Path,sys.argv[1:])
def inspect(path):
 data=path.read_bytes();assert len(data)<32*1024*1024
 assert data[:8]==b'GMSYN001'
 length=struct.unpack_from('<I',data,8)[0];header=json.loads(data[12:12+length]);offset=12+length;tables=[]
 for name in ['tagger','parser']:
  start=offset;buckets,classes,count=struct.unpack_from('<III',data,offset);offset+=12+8*count
  tables.append({'name':name,'buckets':buckets,'classes':classes,'nonzero_weights':count,'table_sha256':hashlib.sha256(data[start:offset]).hexdigest()})
 assert offset==len(data)
 return {'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'header':header,'tables':tables}
b=inspect(baseline/'model.bin');c=inspect(out/'model.bin')
assert b['tables'][0]==c['tables'][0]
r={'baseline':b,'candidate':c,'identical_tagger':True,'binary_bytes':(out/'reading-syntax').stat().st_size,'binary_sha256':hashlib.sha256((out/'reading-syntax').read_bytes()).hexdigest()}
(out/'model-metadata.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k not in ['baseline','candidate']},indent=2))
