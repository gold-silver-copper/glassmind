"""Compile unchanged probe against saved original/candidate facts source; compare list API bytes."""
import os,subprocess,json,hashlib
from pathlib import Path
r=Path('tools/facts/matching');out=r/'run/sets';out.mkdir(exist_ok=True);deps=Path(os.environ['CARGO_TARGET_DIR'])/'release/deps';data=Path('/home/gsc/.cache/glassmind/continuous-20261010/factmatch-source')
def library(name):
 matches=list(deps.glob(f'lib{name}-*.rlib'));assert len(matches)==1,(name,matches);return matches[0]
shared=['--edition=2024','-L',f'dependency={deps}','--extern',f'glassmind_everyday={library("glassmind_everyday")}','--extern',f'serde={library("serde")}']
questions=''.join(row.split('\t',3)[3]+'\n' for row in (data/'valid.tsv').read_text().splitlines()[:200]).encode()
for mode,source in [('baseline',r/'run/baseline/source/src/lib.rs'),('candidate',Path('crates/facts/src/lib.rs'))]:
 lib=out/f'lib{mode}_facts.rlib';binary=out/f'{mode}-sets'
 subprocess.run(['rustc','--crate-name',f'{mode}_facts','--crate-type','rlib',*shared,source,'-o',lib],check=True)
 subprocess.run(['rustc','--crate-name','sets_probe',*shared,'--extern',f'facts={lib}','--extern',f'serde_json={library("serde_json")}',r/'sets_probe.rs','-o',binary],check=True)
 raw=subprocess.run([binary,data/'relation.json',data/'index.json'],input=questions,capture_output=True,check=True).stdout
 (out/f'{mode}.jsonl').write_bytes(raw)
assert (out/'baseline.jsonl').read_bytes()==(out/'candidate.jsonl').read_bytes()
rows=[json.loads(x) for x in raw.splitlines()];report={'rows':len(rows),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'byte_identical':True,'nonempty':sum(x['returned']>0 for x in rows),'multivalued':sum(x['total_snapshot_matches']>1 for x in rows)}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(report)
