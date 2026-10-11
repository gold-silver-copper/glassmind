"""Freeze owned source and complete evidence without copying launcher/private logs."""
import hashlib,json,shutil,subprocess,sys
from pathlib import Path
repo=Path(__file__).resolve().parents[3];run=Path(sys.argv[1]).resolve();dest=run/'frozen-candidate'
if dest.exists():raise SystemExit('refusing to overwrite an existing freeze')
dest.mkdir();(dest/'source/crates').mkdir(parents=True);(dest/'source/tools/facts').mkdir(parents=True)
shutil.copytree(repo/'crates/facts',dest/'source/crates/facts')
shutil.copytree(repo/'tools/facts/temporal',dest/'source/tools/facts/temporal')
shutil.copytree(run/'reports',dest/'reports');shutil.copytree(run/'baseline',dest/'baseline')
shutil.copy2(run/'temporal.json',dest/'temporal.json')
(dest/'binary').mkdir()
for name,path in [('glassmind-facts','debug/glassmind-facts'),('continuous_predictions','debug/examples/continuous_predictions'),('temporal_proof','debug/examples/temporal_proof')]:shutil.copy2(run/'target'/path,dest/'binary'/name)
(dest/'provenance').mkdir()
for name in ['manifest.json','source.json','index-source.json','LICENSE.txt']:shutil.copy2(repo.parent/'factmatch-source'/name,dest/'provenance'/('factmatch-'+name))
shutil.copytree(repo.parent/'temporal-source',dest/'raw-source')
patch=subprocess.check_output(['git','diff','--binary','--','crates/facts','tools/facts/temporal'],cwd=repo)
for name in subprocess.check_output(['git','ls-files','--others','--exclude-standard','--','crates/facts','tools/facts/temporal'],cwd=repo,text=True).splitlines():
 result=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',name],cwd=repo,capture_output=True)
 if result.returncode not in [0,1]:raise RuntimeError(result.stderr.decode())
 patch+=result.stdout
(dest/'candidate.patch').write_bytes(patch)
files={}
for p in sorted(dest.rglob('*')):
 if p.is_file():
  b=p.read_bytes();files[str(p.relative_to(dest))]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
manifest={'source_base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'files':files}
(dest/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
for rel,record in files.items():
 b=(dest/rel).read_bytes();assert len(b)==record['bytes'] and hashlib.sha256(b).hexdigest()==record['sha256'],rel
print(json.dumps({'files':len(files),'bytes':sum(r['bytes'] for r in files.values()),'manifest_sha256':hashlib.sha256((dest/'manifest.json').read_bytes()).hexdigest(),'verified':True},indent=2))
