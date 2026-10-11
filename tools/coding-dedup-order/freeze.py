"""Freeze owned sources and evidence; never inspect unrelated run artifacts."""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);a=p.parse_args();r=a.run.resolve();f=r/'frozen-candidate';f.mkdir(exist_ok=True)
def copy(src,dst):
 if src.is_dir():shutil.copytree(src,dst,dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__'))
 else:dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
copy(r/'frozen-before',f/'original')
copy(Path('crates/world-rust'),f/'final-source/crates/world-rust')
copy(Path('tools/coding-dedup-order'),f/'final-source/tools/coding-dedup-order')
if (f/'glassmind').exists():assert (f/'glassmind').read_bytes()==(r/'target/release/glassmind').read_bytes(), 'evaluated binary differs from build'
copy(r/'target/release/glassmind',f/'glassmind')
copy(r/'baseline/glassmind',f/'baseline-glassmind')
(f/'tracked.diff').write_bytes(subprocess.check_output(['git','diff','--','crates/world-rust']))
# git diff omits untracked additions: provide a complete per-file diff as well.
old=r/'frozen-before/world-rust';new=Path('crates/world-rust')
import difflib
patch=''
for path in sorted(new.rglob('*')):
 if not path.is_file():continue
 rel=path.relative_to(new);original=old/rel
 before=original.read_text().splitlines(keepends=True) if original.exists() else []
 patch+=''.join(difflib.unified_diff(before,path.read_text().splitlines(keepends=True),fromfile=f'a/crates/world-rust/{rel}' if original.exists() else '/dev/null',tofile=f'b/crates/world-rust/{rel}'))
(f/'complete-world-rust.diff').write_text(patch)
for name in ['baseline-full','baseline-sample','candidate-full','candidate-sample','ambiguity-replay','native-evidence','extremum-evidence','native-evidence-attempt1','extremum-evidence-attempt1']:
 if (r/name).exists():
  # These are only this worker's bounded public-source/native artifacts.
  copy(r/name,f/'evidence'/name)
for path in r.glob('*.log'):copy(path,f/'logs'/path.name)
for name in ['comparison.json','audit.json']:
 if (r/name).exists():copy(r/name,f/name)
for path in (r/'native-scratch').glob('*/choice.json'):copy(path,f/'choices'/path.parent.name/'choice.json')
for path in (r/'native-scratch').glob('*/witness.json'):copy(path,f/'choices'/path.parent.name/'witness.json')
source=Path('/home/gsc/.cache/glassmind/continuous-20261010/extremum-source')
copy(source/'manifest.json',f/'TRAIN-source-manifest.json')
metadata=dict(source_head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),new_external_data=False,existing_TRAIN=dict(revision='rustbench f79e71a',license='MIT OR Apache-2.0',files=110,bytes=57265),scope='worker TRAIN fixed27 and public authored native contracts; full765 is coordinator gate',limitation='Native public tests are not proof of intended semantics.')
(f/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
entries=[dict(path=str(path.relative_to(f)),bytes=path.stat().st_size,sha256=hashlib.sha256(path.read_bytes()).hexdigest()) for path in sorted(f.rglob('*')) if path.is_file() and path!=f/'hashes.json']
(f/'hashes.json').write_text(json.dumps(entries,indent=2)+'\n')
for entry in entries:assert hashlib.sha256((f/entry['path']).read_bytes()).hexdigest()==entry['sha256']
print(json.dumps(dict(files=len(entries),bytes=sum(e['bytes'] for e in entries),manifest_sha256=hashlib.sha256((f/'hashes.json').read_bytes()).hexdigest(),binary_sha256=hashlib.sha256((f/'glassmind').read_bytes()).hexdigest()),indent=2))
