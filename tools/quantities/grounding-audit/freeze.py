import hashlib,json,os,shutil,subprocess
from pathlib import Path
ROOT=Path.cwd();O=ROOT/'tools/quantities/grounding-audit';R=Path(os.environ.get('AUDIT_RUN',str(ROOT.parent/'groundingaudit-run/audit')));S=ROOT.parent/'groundingaudit-source';F=R.parent/'frozen-candidate'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def dump(p,x):p.write_text(json.dumps(x,sort_keys=True,indent=2)+'\n')
assert not F.exists(),'preserve existing frozen candidate'
before=json.loads((R/'runtime-before.json').read_text());after={p:sha(Path(p)) for p in before};assert before==after
for item in json.loads((S/'manifest.json').read_text())['files']:
 p=S/item['file'];assert sha(p)==item['sha256'] and p.stat().st_size==item['bytes']
dump(R/'final-integrity.json',dict(runtime_sources_unchanged=True,mounted_source_manifest_unchanged=True,runtime_files=len(before),git_head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),runtime_manifest_sha256=sha(R/'runtime-before.json')))
F.mkdir();shutil.copytree(S,F/'original-train-assets');shutil.copytree(R,F/'audit');shutil.copytree(O,F/'owned-tools')
(F/'binary').mkdir();shutil.copy2(Path(os.environ['CARGO_TARGET_DIR'])/'release/grounding-audit-helper',F/'binary/grounding-audit-helper')
for p in (Path(os.environ['CARGO_TARGET_DIR'])/'release/deps').glob('grounding_audit_helper-*'):
 if p.is_file() and os.access(p,os.X_OK):shutil.copy2(p,F/'binary'/p.name)
files={str(p.relative_to(F)):dict(bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(F.rglob('*')) if p.is_file()}
dump(F/'manifest.json',dict(files=files,total_files=len(files),total_bytes=sum(x['bytes'] for x in files.values())))
# Verify all copies against their newly recorded manifest.
for name,v in files.items():assert sha(F/name)==v['sha256']
dump(O/'freeze-receipt.json',dict(path=str(F),manifest_sha256=sha(F/'manifest.json'),files=len(files),bytes=sum(x['bytes'] for x in files.values()),integrity=json.loads((R/'final-integrity.json').read_text())))
print((O/'freeze-receipt.json').read_text())
