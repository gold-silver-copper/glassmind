"""Freeze only this worker's source and explicitly named public-test artifacts."""
from pathlib import Path
import hashlib,json,shutil,subprocess
root=Path.cwd();run=root.parent/'extremum-run';dest=run/'frozen-candidate';dest.mkdir(exist_ok=True)
for rel in ['crates/world-rust','tools/coding-extremum']:
 shutil.copytree(root/rel,dest/'submission'/rel,dirs_exist_ok=True)
for rel in ['Cargo.toml','Cargo.lock','tools/coding-repair-dev.py']:
 p=dest/'submission'/rel;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/rel,p)
originals=subprocess.check_output(['git','ls-tree','-r','--name-only','HEAD','crates/world-rust'],text=True).splitlines()
for rel in originals:
 p=dest/'original-source'/rel;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(subprocess.check_output(['git','show','HEAD:'+rel]))
(dest/'changes.patch').write_bytes(subprocess.check_output(['git','diff','--','crates/world-rust']))
for name in ['baseline-full','candidate-sample','candidate-full','native','native-final','native-final2','native-final3','native-scratch2','native-scratch3','legacy-replay','legacy-supplement']:
 shutil.copytree(run/name,dest/'evidence'/name,dirs_exist_ok=True)
logs=['baseline-full','fmt','fmt2','fmt3','fmt4','fmt5','fmt6','fmt-final','fmt-check','test1','test2','test3','test-final','test-supplement-final','clippy1','clippy-final','release1','release-final','release-frozen-check','candidate-sample','candidate-full','comparison','legacy-replay','legacy-supplement']
for name in logs:
 shutil.copy2(run/(name+'.log'),dest/'evidence'/(name+'.log'))
shutil.copy2(run/'comparison.json',dest/'evidence/comparison.json')
for rel in ['HANDOFF.md','COMMANDS.md','ATTEMPTS.md']:
 shutil.copy2(root/'tools/coding-extremum'/rel,dest/rel)
files=[]
for p in sorted(dest.rglob('*')):
 if p.is_file() and p!=dest/'manifest.json':
  raw=p.read_bytes();files.append(dict(path=str(p.relative_to(dest)),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
manifest=dict(base=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),status='uncommitted bounded candidate; coordinator DEV gate pending',files=files)
(dest/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
for f in files:
 p=dest/f['path'];assert p.stat().st_size==f['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest()==f['sha256']
print(len(files),sum(f['bytes'] for f in files),'manifest verified')
