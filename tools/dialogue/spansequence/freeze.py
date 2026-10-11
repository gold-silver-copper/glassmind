"""Hash the candidate, source, consumed public data and frozen components before DEV."""
import hashlib,json,pathlib,os,subprocess
run=pathlib.Path(os.environ['TMPDIR'])/'spansequence';base=run.parent.parent/'baseline';root=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full')
def record(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
weights=run/'sequence.unselected.json'
assert weights.stat().st_size<=40*1024*1024
assert not (run/'sequence.json').exists()
assert json.loads((run/'train-report.json').read_text())['selected'] is None
sources=sorted(pathlib.Path('crates/dialogue').rglob('*.rs'))+[pathlib.Path('crates/dialogue/Cargo.toml'),pathlib.Path('Cargo.lock'),pathlib.Path('Cargo.toml')]+sorted(pathlib.Path('tools/dialogue/spansequence').glob('*.py'))+[pathlib.Path('tools/dialogue/spansequence/EXPERIMENT.md')]
report=dict(base_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),source_revision='e852981ae34990f4358979625854259302feaa78',license='CC-BY-SA-4.0',download_bytes=0,source=[record(p) for p in sources],data=[record(root/p) for p in ['selected/train.json','selected/dev.json','train/schema.json','selected/dev-schema.json','LICENSE.txt']],models=[record(p) for p in [base/'model.json',weights]],baseline_predictions=[record(base/p/'predictions.json') for p in ['first','full']],evaluators=[record(pathlib.Path(p)) for p in ['tools/dialogue/evaluate.py','tools/dialogue/compare.py','crates/dialogue/tools/upstream_metrics.py']],binaries=[record(pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release'/p) for p in ['dialogue-multi','dialogue-spansequence-train','dialogue-spansequence-probe','glassmind-dialogue']],decision="rejected CAL; no candidate model assembled; baseline DEV replay only")
report['source_sha256']=hashlib.sha256(json.dumps(report['source'],sort_keys=True).encode()).hexdigest()
(run/'freeze.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(source_sha256=report['source_sha256'],models=report['models']),indent=2))
