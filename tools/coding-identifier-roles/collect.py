"""Collect retained run evidence without opening any other worker's logs."""
import hashlib,json,os,shutil,subprocess
from pathlib import Path
run=Path(os.environ['TMPDIR']).parent
out=Path(__file__).parent
reports=out/'reports'
for name in ['controls','baseline-sample','baseline-full','candidate-sample','candidate-full','candidate-v2-sample','candidate-v2-full']:
 shutil.copy2(run/name/'results.json',reports/(name+'.json'))
for mode in ['baseline','candidate']:
 for folder in ['regressions','addfield']:
  origin=run/(folder+'-'+mode); dest=reports/(folder+'-'+mode);dest.mkdir(exist_ok=True)
  for file in origin.rglob('*'):
   if file.is_file() and (file.name=='results.json' or file.name=='status.txt' or file.suffix=='.log'):
    target=dest/file.relative_to(origin);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(file,target)
base=json.loads((reports/'baseline-full.json').read_text());candidate=json.loads((reports/'candidate-v2-full.json').read_text())
cases={c['id']:c for c in json.loads((out/'contracts.json').read_text())}
audit=[]
for b,c in zip(base,candidate):
 assert b['id']==c['id']
 assert not b['pass_'] or c['pass_']
 assert c['pass_']
 root=run/'candidate-v2-full'/c['id']
 actual_paths=sorted(str(p.relative_to(root)) for p in root.rglob('*') if p.is_file())
 expected_paths=sorted(list(cases[c['id']]['files'])+(['Cargo.lock'] if (root/'Cargo.lock').exists() else []))
 assert actual_paths==expected_paths,(c['id'],actual_paths)
 if b['result']['stdout']!=c['result']['stdout'] or b['actual']!=c['actual']:
  audit.append(dict(id=c['id'],baseline_pass=b['pass_'],candidate_pass=c['pass_'],baseline_reply=b['result']['stdout'],candidate_reply=c['result']['stdout'],changed_paths=[p for p in c['actual'] if c['actual'][p]!=cases[c['id']]['files'][p]],exact=c['exact'],behavior=c['behavior'],files=c['actual'],native_test_ran=c['changed'] and '1 passed' in c['after']['stdout']))
summary=dict(total=len(base),baseline_pass=sum(r['pass_'] for r in base),candidate_pass=sum(r['pass_'] for r in candidate),retained=sum(b['pass_'] and c['pass_'] for b,c in zip(base,candidate)),new_successes=[c['id'] for b,c in zip(base,candidate) if not b['pass_'] and c['pass_']],baseline_renames=sum(r['changed'] for r in base),candidate_renames=sum(r['changed'] for r in candidate),all_negative_exact=all(c['actual']==cases[c['id']]['files'] for c in candidate if not cases[c['id']]['positive']),all_inventory_exact=True,changed_cases=audit)
(reports/'comparison.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
def identity(p):
 return dict(path=str(p),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())
source_files=[Path('crates/english/src/lexicon.rs'),Path('crates/english/src/sem.rs'),Path('crates/english-rust/src/lib.rs'),Path('crates/english-rust/src/identifier_roles.rs')]
ids=[identity(p) for p in source_files]+[identity(run/p) for p in ['baseline/glassmind','glassmind-rejected-v1','glassmind-candidate-v2']]
ids+=[identity(Path('tools')/(name+'.py')) for name in ['coding-metatypes-dev','coding-answerfiles-dev','coding-answerfiles-regression','coding-addfields-dev','coding-edits-dev']]
(reports/'identities.json').write_text(json.dumps(ids,indent=2)+'\n')
provenance=dict(source='Locally authored public development contracts; no downloads, models, benchmark data or hidden inputs',version='identifier-roles-v1, 2026-10-11; frozen at fd01b9bdc2ff68402841577fb3510422e6e47ec6 before runtime edits',license='MIT OR Apache-2.0 (workspace license)',data=identity(out/'contracts.json'),generator=identity(out/'author.py'),cases=len(cases),expanded_fixture_bytes=sum(len(t.encode()) for c in cases.values() for t in c['files'].values()),request_bytes=sum(len(c['request'].encode()) for c in cases.values()),supplemental='owner-audit.py is post-implementation diagnosis, not frozen-gate gain')
(out/'data-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k!='changed_cases'},indent=2))
