"""Final immutable handoff; refuses existing destination and verifies copied bytes."""
import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
ROOT=Path.cwd();RUN=ROOT.parent/'equivbinding-run';R=Path(os.environ.get('EQUIV_RUN',str(RUN/'experiment')));S=ROOT.parent/'groundingaudit-source';O=ROOT/'tools/quantities/equivalent-binding';F=RUN/'frozen-candidate'
assert not F.exists(),'preserve existing freeze; use a separately named revision'
assert json.loads((R/'results.json').read_text())['decision']=='REJECT'
assert json.loads((R/'independent-verification.json').read_text())['passed']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
F.mkdir();shutil.copytree(S,F/'original');shutil.copytree(R,F/'experiment');shutil.copytree(O,F/'adapter')
shutil.copy2(Path(os.environ['CARGO_TARGET_DIR'])/'release/equivalent-binding-helper',F/'equivalent-binding-helper')
# Unified patch of owned additions, without touching the index or tracked files.
patch=[]
for p in sorted(O.iterdir()):
 if p.is_file():
  r=subprocess.run(['git','diff','--no-index','--','/dev/null',str(p.relative_to(ROOT))],capture_output=True,check=False)
  assert r.returncode in [0,1];patch.append(r.stdout)
(F/'owned-additions.patch').write_bytes(b''.join(patch))
meta={'base_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'git_status':subprocess.check_output(['git','status','--short'],text=True),'python':sys.version,'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'scope':'one TRAIN-only rejected fixed matched comparison; no deployment or selected answers','manifest_excludes':'manifest.json itself; receipt is external to freeze'}
(F/'metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
for a,b in [(S,F/'original'),(R,F/'experiment'),(O,F/'adapter')]:
 for p in a.rglob('*'):
  if p.is_file():assert p.read_bytes()==(b/p.relative_to(a)).read_bytes()
manifest={str(p.relative_to(F)):{'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(F.rglob('*')) if p.is_file()}
(F/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
for rel,m in manifest.items():assert sha(F/rel)==m['sha256'] and (F/rel).stat().st_size==m['bytes']
receipt={'directory':str(F),'manifest_sha256':sha(F/'manifest.json'),'files':len(manifest),'bytes':sum(m['bytes'] for m in manifest.values()),'decision':'REJECT','complete':True,'verification':'all copied files byte-exact; every manifest entry rehashed'}
(RUN/'freeze-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');(O/'freeze-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
