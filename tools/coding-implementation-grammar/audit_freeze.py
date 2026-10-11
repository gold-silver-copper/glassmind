"""Post-archive audit: forbidden grammar expansion, not a changed original gate."""
from pathlib import Path
import shutil,subprocess,json,hashlib,os,sys
R=Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run');out=R/(sys.argv[1] if len(sys.argv)>1 else 'freeze-audit');out.mkdir(exist_ok=False);cases=['collect_values needs a body, implement it','collect_values needs implementing, implement it'];rows=[]
for i,text in enumerate(cases):
 root=out/str(i);(root/'src').mkdir(parents=True)
 for name in ['Cargo.toml','src/lib.rs']:shutil.copy(R/'baseline-native/sum'/name,root/name)
 def snap():return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()}
 before=snap();env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch');p=subprocess.run([str(R/'frozen-candidate/glassmind'),'do',str(root),text],env=env,capture_output=True,text=True,timeout=110);(out/f'{i}.log').write_text(p.stdout+p.stderr)
 rows.append(dict(text=text,allowed_by_frozen_accepted_grammar=False,accepted='Applied test-verified proposal' in p.stdout,before=before,after=snap(),stdout=p.stdout,exit=p.returncode))
(out/'results.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2));assert all(r['accepted'] for r in rows)
# Exit zero means the contract mismatch was reproduced, not acceptance.
