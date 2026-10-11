#!/usr/bin/env python3
"""Archive and verify all owned evidence before restoring rejected production source."""
import hashlib,json,pathlib,shutil,subprocess
P=pathlib.Path;root=P(__file__).resolve().parents[3];run=root.parent/'graphsyntax-run';own=root/'tools/reading/syntax-graph'
def ident(p):return dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())
archive=run/'frozen-candidate';assert not archive.exists()
tracked=['crates/reading/src/syntax.rs','crates/reading/src/bin/reading-syntax.rs']
patch=subprocess.check_output(['git','diff','--src-prefix=a/','--dst-prefix=b/','--',*tracked],cwd=root)
for name in ['crates/reading/src/syntax_graph.rs',*[str(p.relative_to(root)) for p in sorted(own.iterdir()) if p.is_file() and p.name not in ['rejected.patch','ARCHIVE.json']]]:
    result=subprocess.run(['git','diff','--no-index','--src-prefix=a/','--dst-prefix=b/','/dev/null',name],cwd=root,capture_output=True)
    assert result.returncode==1;patch+=result.stdout
(own/'rejected.patch').write_bytes(patch)
archive.mkdir()
for name in ['original','baseline','candidate']:shutil.copytree(run/name,archive/name)
shutil.copytree(own,archive/'tools')
for p in run.iterdir():
    if p.is_file() and (p.suffix=='.log' and p.name not in ['stderr.log'] or p.name in ['FREEZE.json','prefit.json']):shutil.copy2(p,archive/p.name)
manifest={str(p.relative_to(archive)):ident(p) for p in sorted(archive.rglob('*')) if p.is_file()}
manifest_path=run/'frozen-candidate-manifest.json';manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
for rel,value in manifest.items():assert ident(archive/rel)==value
(own/'ARCHIVE.json').write_text(json.dumps(dict(path=str(archive),files=len(manifest),bytes=sum(x['bytes'] for x in manifest.values()),manifest=str(manifest_path),manifest_identity=ident(manifest_path)),indent=2)+'\n')
# Failed gate only: restore exact original production files after verification.
score=json.loads((run/'candidate/audit/score.json').read_text())['groups']['all']
assert score['las_correct']<19673 or score['uas_correct']<20540
for name,target in [('syntax.rs','crates/reading/src/syntax.rs'),('reading-syntax.rs','crates/reading/src/bin/reading-syntax.rs')]:
    shutil.copy2(run/'original'/name,root/target)
    assert (root/target).read_bytes()==(run/'original'/name).read_bytes()
(root/'crates/reading/src/syntax_graph.rs').unlink()
(own/'rollback.json').write_text(json.dumps({p:ident(root/p) for p in tracked},indent=2)+'\n')
print(json.dumps(dict(files=len(manifest),bytes=sum(x['bytes'] for x in manifest.values()),restored=True)))
