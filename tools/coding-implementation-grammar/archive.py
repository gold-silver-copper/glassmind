"""Archive only this worker's allowlisted sources, data, commands and evidence."""
from pathlib import Path
import json,hashlib,shutil,subprocess
T=Path('tools/coding-implementation-grammar');R=Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run');F=R/'frozen-candidate'
def identity(p):return dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())
fixture_names=['native.json','boundaries.json','grammar.json','contracts.rs']
(T/'data-provenance.json').write_text(json.dumps(dict(license='MIT OR Apache-2.0',authored_date='2026-10-11',files={n:identity(T/n) for n in fixture_names}),indent=2)+'\n')
# Preserve the incidental, worker-created bytecode before removing it.
pyc=Path('tools/coding-implementation-routes/original/__pycache__/fixtures.cpython-314.pyc')
if pyc.exists():
 dst=F/'original/incidental-fixtures.cpython-314.pyc';shutil.copy(pyc,dst);assert identity(pyc)==identity(dst)
 (T/'reports/incidental-cache.json').write_text(json.dumps(dict(path=str(pyc),archived=str(dst),identity=identity(dst),created='2026-10-11T14:39:12Z',cleanup='only this imported-fixture bytecode file'),indent=2)+'\n')
for name in ['baseline-boundaries','baseline-boundaries-complete','baseline-native','baseline-train-sample','candidate-boundaries','candidate-sample','candidate-native','candidate-sample-verification','candidate-native-verification','environment','final-boundaries','final-sample','final-native','environment-final','train-sample','train-sample-verification','train-full','train-full-verification','distinct-witness','release-sample','release-boundaries','package-binding','package-binding-scratch','revision1']:
 shutil.copytree(R/name,F/'raw'/name,dirs_exist_ok=True,ignore=shutil.ignore_patterns('implement-target','bin','__pycache__'))
for p in (R/'test-scratch').glob('implement*.evidence'):
 shutil.copytree(p,F/'raw/test-scratch'/p.name,dirs_exist_ok=True)
for name in ['parser-audit','package-probe']:shutil.copy(R/name,F/name)
# Do not include cache files or any private worker/session logs.
for target in [F/'tools-final',F/'source/tools/coding-implementation-grammar']:
 shutil.copytree(T,target,dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__'))
tracked=['crates/cli/src/main.rs','crates/english-rust/src/lib.rs','crates/world-rust/src/lib.rs']
patch=subprocess.run(['git','diff','--binary','--src-prefix=a/','--dst-prefix=b/','--',*tracked],capture_output=True,check=True).stdout
new=[Path('crates/english-rust/src/implementation_grammar.rs'),Path('crates/world-rust/src/implementation.rs')]+sorted(p for p in T.rglob('*') if p.is_file() and '__pycache__' not in p.parts)
for p in new:
 diff=subprocess.run(['git','diff','--no-index','--binary','--src-prefix=a/','--dst-prefix=b/','--','/dev/null',str(p)],capture_output=True)
 assert diff.returncode==1,(p,diff.stderr);patch+=diff.stdout
(F/'candidate.patch').write_bytes(patch)
for p in (F/'source/crates').rglob('*'):
 if p.is_file():assert p.read_bytes()==Path(p.relative_to(F/'source')).read_bytes()
for line in (T/'FROZEN.sha256').read_text().splitlines():
 digest,name=line.split('  ',1);assert identity(T/name)['sha256']==digest
index={str(p.relative_to(F)):identity(p) for p in F.rglob('*') if p.is_file() and p.name not in ['MANIFEST.json','archive-verification.json']}
for name,data in index.items():assert identity(F/name)==data
(F/'archive-verification.json').write_text(json.dumps(dict(verified=True,payload_files=len(index),payload_bytes=sum(x['bytes'] for x in index.values()),patch=identity(F/'candidate.patch'),source_matches_worktree=True,frozen_contract_hashes_match=True,excluded_from_hash_recursion='MANIFEST.json only'),indent=2)+'\n')
index['archive-verification.json']=identity(F/'archive-verification.json');(F/'MANIFEST.json').write_text(json.dumps(index,indent=2)+'\n')
for name,data in index.items():assert identity(F/name)==data
# Archive verified before cleanup; never remove an unrelated cache or target.
if pyc.exists():
 assert identity(pyc)==identity(F/'original/incidental-fixtures.cpython-314.pyc');pyc.unlink()
 try:pyc.parent.rmdir()
 except OSError:pass
print(json.dumps(dict(files=len(index),bytes=sum(x['bytes'] for x in index.values()),manifest=identity(F/'MANIFEST.json'),patch=identity(F/'candidate.patch')),indent=2))
