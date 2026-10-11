"""Archive exact submission after every owned validation process completes."""
import hashlib,json,shutil,subprocess,datetime
from pathlib import Path
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');R=W/'timedmatch-run';F=R/'frozen-candidate'
assert not F.exists(), 'do not overwrite an original submission'
assert (R/'evidence/cli/report.json').exists(), 'full CLI replay still pending'
F.mkdir();(F/'source').mkdir();(F/'bin').mkdir()
for rel in ['crates/facts','tools/facts/timed-matching']:
    shutil.copytree(rel,F/'source'/rel,ignore=shutil.ignore_patterns('__pycache__'))
for rel in ['Cargo.toml','Cargo.lock']:
    shutil.copy2(rel,F/'source'/rel)
for rel in ['glassmind','glassmind-facts','examples/timed_matching_proof','examples/continuous_predictions','examples/temporal_proof']:
    shutil.copy2(R/'target/release'/rel,F/'bin'/Path(rel).name)
shutil.copytree(R/'evidence',F/'reports')
(F/'tracked.patch').write_bytes(subprocess.check_output(['git','diff','--binary','--','crates/facts']))
shutil.copy2(R/'baseline/manifest.json',F/'baseline-input-manifest.json')
(F/'status.txt').write_bytes(subprocess.check_output(['git','status','--short']))
for name,source in [('factmatch',W/'factmatch-source'),('temporal',W/'temporal-source'),('timedmatch',W/'timedmatch-source')]:
    shutil.copy2(source/'manifest.json',F/(name+'-source-manifest.json'))
files=[]
for p in sorted(F.rglob('*')):
    if p.is_file():
        b=p.read_bytes();files.append(dict(path=str(p.relative_to(F)),bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
baseline_files=[]
for name in ['glassmind','glassmind-facts','continuous_predictions']:
    p=R/'baseline'/name;b=p.read_bytes();baseline_files.append(dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
manifest=dict(baseline_files=baseline_files,base=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),source_uncommitted=True,model='Requested Astra fast, ChatGPT-only; no model subprocess calls',files=files)
(F/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
for f in files:
    assert hashlib.sha256((F/f['path']).read_bytes()).hexdigest()==f['sha256']
print(json.dumps(dict(files=len(files),bytes=sum(f['bytes'] for f in files),manifest_sha256=hashlib.sha256((F/'manifest.json').read_bytes()).hexdigest(),binary_sha256={f['path']:f['sha256'] for f in files if f['path'].startswith('bin/')})))
