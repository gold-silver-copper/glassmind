"""Freeze owned candidate source/probes/binaries before final native contracts."""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--target',type=Path,required=True);a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=False)
files=sorted(list(Path('crates/facts').rglob('*.rs'))+list(Path('tools/facts/verification').rglob('*')))
files=[f for f in files if f.is_file() and '__pycache__' not in f.parts]
for f in files:
    dest=a.output/'source'/f;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dest)
patch=subprocess.run(['git','diff','--binary','--','crates/facts'],capture_output=True,check=True).stdout
for f in files:
    tracked=subprocess.run(['git','ls-files','--error-unmatch',str(f)],capture_output=True).returncode==0
    if not tracked:
        r=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',str(f)],capture_output=True)
        assert r.returncode in (0,1);patch+=r.stdout
(a.output/'source.patch').write_bytes(patch)
for name,rel in [('glassmind-facts','release/glassmind-facts'),('continuous_predictions','release/examples/continuous_predictions')]:
    shutil.copy2(a.target/rel,a.output/name)
manifest={str(f.relative_to(a.output)):{'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted(a.output.rglob('*')) if f.is_file()}
(a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(f'Archived {len(manifest)} files to {a.output}')
