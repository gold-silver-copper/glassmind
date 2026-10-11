"""Freeze only owned source/tools and this batch's evidence; verify every copied file."""
import hashlib,json,os,shutil,subprocess
from pathlib import Path
root=Path.cwd();tools=root/'tools/facts/matching';run=tools/'run';frozen=Path('/home/gsc/.cache/glassmind/continuous-20261010/factmatch-run/frozen-candidate')
assert not frozen.exists(), 'do not overwrite an existing frozen candidate'
frozen.mkdir()
def copy(src,dst):
 dst=frozen/dst;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
 assert src.read_bytes()==dst.read_bytes()
for p in (root/'crates/facts').rglob('*'):
 if p.is_file():copy(p,Path('source')/p.relative_to(root))
copy(root/'Cargo.lock','source/Cargo.lock')
for p in tools.iterdir():
 if p.is_file():copy(p,Path('tools/facts/matching')/p.name)
for p in run.rglob('*'):
 if not p.is_file():continue
 rel=p.relative_to(run)
 # Keep all reports/JSON predictions and saved original source, not scratch builds.
 if p.suffix in ['.json','.jsonl','.log','.rs','.toml'] or (str(rel).startswith('baseline/source/') and p.is_file()):
  copy(p,Path('evidence')/rel)
copy(Path(os.environ['CARGO_TARGET_DIR'])/'release/glassmind-facts','binary/glassmind-facts')
copy(Path(os.environ['CARGO_TARGET_DIR'])/'release/examples/continuous_predictions','binary/continuous_predictions')
copy(run/'baseline/glassmind-facts','baseline/glassmind-facts')
copy(run/'baseline/continuous_predictions','baseline/continuous_predictions')
patch=subprocess.check_output(['git','diff','--binary','--','Cargo.lock','crates/facts'])
for p in ['crates/facts/src/matching.rs','crates/facts/tests/matching.rs']:
 result=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',p],capture_output=True);assert result.returncode==1;patch+=result.stdout
(frozen/'candidate.patch').write_bytes(patch)
manifest={}
for p in sorted(frozen.rglob('*')):
 if p.is_file():
  b=p.read_bytes();manifest[str(p.relative_to(frozen))]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
(frozen/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
for name,m in manifest.items():
 b=(frozen/name).read_bytes();assert len(b)==m['bytes'] and hashlib.sha256(b).hexdigest()==m['sha256']
(run/'frozen-candidate').symlink_to(frozen,target_is_directory=True)
summary={'frozen':str(frozen),'files':len(manifest),'bytes':sum(x['bytes'] for x in manifest.values()),'manifest_sha256':hashlib.sha256((frozen/'manifest.json').read_bytes()).hexdigest(),'verified':True}
(run/'freeze-verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
