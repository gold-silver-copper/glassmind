"""Preregistered loader fixtures, including valid JSON beyond each byte cap."""
from pathlib import Path
import hashlib,json,shutil
base=Path(__file__).parent;source=base.parents[1]/'draftsession-source';root=base/'bundles';root.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();made=[]
manifest=dict(version=1,model_sha256=sha(source/'model.json'),schema_sha256=sha(source/'schema.json'),source='SGD utterance-only trained three-service snapshot e852981ae34990f4358979625854259302feaa78',license='CC-BY-SA-4.0',scope='Provisional learned dialogue state; restricted three-service development slice')
for name in ['valid',*json.loads((base/'loader-contracts.json').read_text())['invalid_bundles'],'model_version','model_rebound_same_semantics','schema_rebound_same_semantics']:
 d=root/name;d.mkdir();m=manifest.copy()
 for file in ['model.json','schema.json']:(d/file).symlink_to(source/file)
 if name=='wrong_version':m['version']=2
 elif name=='unknown_manifest_field':m['frames']=[]
 elif name.startswith('empty_'):m[name.removeprefix('empty_')]=' '
 elif name in ['missing_model','missing_schema']:(d/(name.removeprefix('missing_')+'.json')).unlink()
 elif name in ['model_hash_mismatch','schema_hash_mismatch']:m[name.replace('_hash_mismatch','_sha256')]='0'*64
 elif name=='schema_model_incompatible':
  p=d/'schema.json';p.unlink();p.write_text(json.dumps(json.loads((source/'schema.json').read_text())[:-1]));m['schema_sha256']=sha(p)
 elif name=='model_version':
  v=json.loads((source/'model.json').read_text());v['version']=999;p=d/'model.json';p.unlink();p.write_text(json.dumps(v));m['model_sha256']=sha(p)
 elif name in ['model_valid_json_over_cap','schema_valid_json_over_cap']:
  file='model.json' if name.startswith('model') else 'schema.json';cap=67108864 if name.startswith('model') else 1048576;p=d/file;p.unlink();shutil.copy2(source/file,p)
  with p.open('ab') as f:f.write(b' '*(cap+1-p.stat().st_size))
  assert p.stat().st_size==cap+1;json.loads(p.read_text());m[file.replace('.json','_sha256')]=sha(p)
 elif name in ['model_rebound_same_semantics','schema_rebound_same_semantics']:
  file='model.json' if name.startswith('model') else 'schema.json';p=d/file;p.unlink();p.write_bytes((source/file).read_bytes()+b' ');m[file.replace('.json','_sha256')]=sha(p)
 if name!='missing_manifest':
  p=d/'manifest.json';p.write_text(json.dumps(m))
  if name=='manifest_valid_json_over_cap':
   with p.open('ab') as f:f.write(b' '*(65537-p.stat().st_size))
   assert p.stat().st_size==65537;json.loads(p.read_text())
 made.append(dict(name=name,files={p.name:dict(bytes=p.stat().st_size,sha256=sha(p),symlink=p.is_symlink()) for p in sorted(d.iterdir())}))
(base/'bundle-fixtures.json').write_text(json.dumps(made,indent=2)+'\n');print(len(made),'bundles prepared and hashed; valid over-cap JSON checked independently')
