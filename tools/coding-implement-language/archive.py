#!/usr/bin/env python3
"""Archive only this worker's five runtime files and owned evidence, before restore."""
import hashlib,json,pathlib,shutil,subprocess
base=pathlib.Path(__file__).resolve().parent
run=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/impllanguage-run')
files=['crates/cli/src/main.rs','crates/english-rust/src/lib.rs','crates/world-rust/src/lib.rs','crates/english-rust/src/implement.rs','crates/world-rust/src/implement.rs']
def entry(p):return dict(sha256=hashlib.sha256(p.read_bytes()).hexdigest(),bytes=p.stat().st_size)
if (base/'rejected.patch').exists():raise SystemExit('Archive already exists; refusing to overwrite evidence')
patch=subprocess.check_output(['git','diff','--',*files[:3]])
for f in files[3:]:
 r=subprocess.run(['git','diff','--no-index','--','/dev/null',f],capture_output=True)
 assert r.returncode==1
 patch+=r.stdout
(base/'rejected.patch').write_bytes(patch)
for f in files:
 dest=base/'rejected-source'/f;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dest)
 assert entry(pathlib.Path(f))==entry(dest)
artifacts={}
for n in ['glassmind-archived','glassmind-candidate','glassmind-final','parser-probe','parser-probe-final']:
 artifacts[n]=entry(run/n)
for n in ['archived-sample','sample','contracts','train','original-archived-sample','original-sample','original-full','final-sample','final-train','final-contracts','final-original-sample','final-original-full','environment','archived-no-target','archived-wrong-action']:
 p=run/n/'results.json'
 if p.exists():shutil.copyfile(p,base/'reports'/(n+'-results.json'))
for n in ['archived-sample','sample','contracts','train','original-archived-sample','original-sample','original-full','final-sample','final-train','final-contracts','final-original-sample','final-original-full','environment','archived-no-target','archived-wrong-action','frozen-train','rust-scratch','final-rust-scratch']:
 for p in (run/n).rglob('*'):
  if any(c in ['target','implement-target','__pycache__'] for c in p.relative_to(run).parts):continue
  if p.is_file() and not p.is_symlink():artifacts[str(p.relative_to(run))]=entry(p)
(base/'reports/raw-artifacts.json').write_text(json.dumps(dict(root=str(run),files=artifacts),indent=2)+'\n')
provenance=json.loads((base/'reports/train-input-provenance.json').read_text())
unique={r['source']:r['files'] for r in provenance}
summary=dict(decision='REJECTED: strict CLI contract suite 33/34; runtime must be restored after archiving',base_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
 gate=dict(original20='20/20',parser_contracts='34/34',environment='6/6',cli_contracts='33/34',required_cli_contracts='34/34',train_test_verified='3/60',distinct_functions=1,styles=['chatty','hurried','terse'],function='sum_even',parent_full765='NOT RUN; no >=177 or retained174 claim'),
 parser_coverage='23/60 only, not implementation correctness',train_refusals='57/57 unchanged',rust_tests='76/76',fmt='passed',clippy='passed',debug_build='passed',release_build='not run: candidate rejected',
 baseline=dict(old_candidate_rejection='UNCHANGED: original 0/60 request coverage',replayed_old_sample='3/3 authored implementations',replayed_original_train_sample='0/1',no_cargo_target='refused unchanged: existing CARGO_TARGET_DIR required',wrong_action='same Cargo.lock/target creation reproduced on archived baseline'),
 data=dict(new_download_bytes=0,authored_license='MIT OR Apache-2.0 (workspace code license)',original_fixture=entry(base/'original/fixtures.py'),new_language_contracts=entry(base/'contracts.json'),train_requests=60,train_request_utf8_bytes=sum(len(r['text'].encode()) for r in provenance),public_projects=len(unique),public_project_files=sum(len(v) for v in unique.values()),public_project_bytes=sum(f['bytes'] for v in unique.values() for f in v.values()),public_source_license='Unestablished in inspected Cargo/src allowlist; no license inferred. Raw project sources retained outside git.',public_source_version='Existing train-v3 and cached rb-implement public production workspaces, exact hashes in train-input-provenance.json; inherited checkpoint benchmark pin f79e71a2a26fdaf15f9bc55225a038b61e66fafb not independently reverified'),
 runtime_source={f:entry(pathlib.Path(f)) for f in files},patch=entry(base/'rejected.patch'),binaries={k:v for k,v in artifacts.items() if '/' not in k})
(base/'reports/validation.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
