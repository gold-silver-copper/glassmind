#!/usr/bin/env python3
"""Capture/hash-verify owned evidence before removing the two offline Rust files."""
import ast, hashlib, json, pathlib, subprocess, tarfile
root=pathlib.Path('tools/dialogue/entity-ledger');artifact=root/'artifacts'
source=[pathlib.Path('crates/dialogue/examples/entity-ledger.rs'),pathlib.Path('crates/dialogue/src/entity_ledger_experiment.rs')]
assert all(p.is_file() for p in source)
for p in root.glob('*.py'):ast.parse(p.read_text())
assert (artifact/'baseline-train.json').read_bytes()==(artifact/'rebuilt-baseline-train.json').read_bytes()
patch=b''
for p in source:
 result=subprocess.run(['git','diff','--no-index','--','/dev/null',str(p)],capture_output=True)
 assert result.returncode==1;patch+=result.stdout
(root/'rejected-source.patch').write_bytes(patch)
paths=sorted(source+[p for p in root.rglob('*') if p.is_file() and p.name not in {'pre-rollback.tar.gz','pre-rollback-manifest.json','archive-verification.json','restored-source.json','final-identities.json'}])
manifest={str(p):{'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths}
(artifact/'pre-rollback-manifest.json').write_text(json.dumps(manifest,indent=2))
archive=artifact/'pre-rollback.tar.gz'
with tarfile.open(archive,'w:gz',compresslevel=1) as tar:
 for p in paths:tar.add(p,arcname=str(p),recursive=False)
 tar.add(artifact/'pre-rollback-manifest.json',arcname=str(artifact/'pre-rollback-manifest.json'))
with tarfile.open(archive,'r:gz') as tar:
 for name,identity in manifest.items():
  f=tar.extractfile(name);assert f is not None;contents=f.read();assert len(contents)==identity['bytes'];assert hashlib.sha256(contents).hexdigest()==identity['sha256']
  assert pathlib.Path(name).read_bytes()==contents
report={'status':'VERIFIED_BEFORE_REMOVAL','files':len(manifest),'bytes':sum(x['bytes'] for x in manifest.values()),'archive_bytes':archive.stat().st_size,'archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'manifest_sha256':hashlib.sha256((artifact/'pre-rollback-manifest.json').read_bytes()).hexdigest()}
(root/'reports/archive-verification.json').write_text(json.dumps(report,indent=2))
# Remove ONLY the two newly introduced experiment files, after their archive check.
for p in source:
 assert p.read_bytes()==(root/'native'/p.name).read_bytes();p.unlink()
original=json.loads((root/'reports/original-source.json').read_text())
for name,h in original.items():assert hashlib.sha256(pathlib.Path(name).read_bytes()).hexdigest()==h
original_files=0
with tarfile.open(artifact/'original-source.tar') as tar:
 for member in tar.getmembers():
  if not member.isfile():continue
  f=tar.extractfile(member);assert f is not None;assert pathlib.Path(member.name).read_bytes()==f.read();original_files+=1
(root/'reports/restored-source.json').write_text(json.dumps({'status':'BYTE_EXACT','original_rust_and_manifest_files':len(original),'all_original_archive_files':original_files,'removed_only':[str(p) for p in source],'production_runtime_modified':False,'original_hashes':original},indent=2))
print(json.dumps(report,indent=2));print('original source verified, offline crate files removed')
