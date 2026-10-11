#!/usr/bin/env python3
"""Archive all owned evidence and verify each member's exact bytes."""
import hashlib,json,pathlib,subprocess,tarfile
H=pathlib.Path(__file__).resolve().parent;root=H.parents[2]
example=root/'crates/dialogue/examples/context-ledger.rs'
# Source-only patch accompanies complete files/artifacts (large raw data stays as files).
code=[example]+sorted(p for p in H.rglob('*') if p.is_file() and 'artifacts' not in p.parts and 'originals' not in p.parts and p.suffix in {'.rs','.py','.sh','.md','.txt','.log'} and p.name not in {'source.patch'})
with open(H/'source.patch','w') as out:
 for p in code:
  result=subprocess.run(['git','diff','--no-index','--','/dev/null',str(p.relative_to(root))],cwd=root,text=True,capture_output=True)
  assert result.returncode in (0,1);out.write(result.stdout)
exclude={'archive.tar.gz','archive-manifest.json','archive-verification.json'}
files=[example]+sorted(p for p in H.rglob('*') if p.is_file() and p.name not in exclude and '__pycache__' not in p.parts)
manifest={}
for p in files:
 b=p.read_bytes();manifest[str(p.relative_to(root))]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
(H/'archive-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
archive=H/'archive.tar.gz'
with tarfile.open(archive,'w:gz',compresslevel=1) as tar:
 for p in files:tar.add(p,arcname=str(p.relative_to(root)),recursive=False)
 tar.add(H/'archive-manifest.json',arcname=str((H/'archive-manifest.json').relative_to(root)),recursive=False)
verified=0
with tarfile.open(archive,'r:gz') as tar:
 for name,record in manifest.items():
  f=tar.extractfile(name);assert f is not None;h=hashlib.sha256();n=0
  while b:=f.read(1024*1024):n+=len(b);h.update(b)
  assert n==record['bytes'] and h.hexdigest()==record['sha256'];verified+=1
h=hashlib.sha256()
with open(archive,'rb') as f:
 while b:=f.read(1024*1024):h.update(b)
r={'verified_members':verified,'payload_bytes':sum(v['bytes'] for v in manifest.values()),'archive_bytes':archive.stat().st_size,'archive_sha256':h.hexdigest(),'every_manifest_member_verified':True}
(H/'archive-verification.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
