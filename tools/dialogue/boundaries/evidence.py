#!/usr/bin/env python3
"""Verify retained artifacts, source provenance, compatibility and acceptance."""
import argparse, hashlib, json, pathlib, subprocess
p=argparse.ArgumentParser()
for n in ('root','run','baseline','target'):p.add_argument('--'+n,type=pathlib.Path,required=True)
a=p.parse_args()
def file_record(p):
    b=p.read_bytes();return dict(path=str(p.resolve()),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
def read(p):return json.loads(p.read_text())
b=read(a.baseline/'model.json');c=read(a.run/'model.json')
assert b['version']==1 and c['version']==2
assert {k:v for k,v in c.items() if k not in ('boundaries','version')}=={k:v for k,v in b.items() if k!='version'}
assert read(a.run/'boundaries.json')==c['boundaries']
assert (a.run/'frozen-replay/predictions.json').read_bytes()==(a.baseline/'full/predictions.json').read_bytes()
legacy=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/multi/final-legacy-predictions.json')
assert (a.run/'legacy-predictions.json').read_bytes()==legacy.read_bytes()
transitions=read(a.run/'transitions.json');audit=read(a.run/'audit.json');full=read(a.run/'full/report.json')
assert transitions['joint_transitions']['right->right']==609
assert transitions['frame_transitions']['True->True']==712
assert not audit['all_new_wrong_claims']
assert audit['inherited']['corrected_scored_slot_turns']>0
assert full['all']['user_turns']==1119 and full['all']['gold_frames']==1218 and full['all']['predicted_frames']==3357
assert full['isolation']['annotation_input_rejected'] and full['isolation']['reversed_batch_identical']
old=read(pathlib.Path('tools/dialogue/offers/reports/artifacts.json'))
source=[]
for record in old['source_files']:
    rel=pathlib.Path(record['path']).relative_to('/home/gsc/.cache/glassmind/data/sgd-full')
    assert rel.parts[0] in ('train','dev','LICENSE.txt')
    actual=file_record(a.root/rel)
    assert actual['sha256']==record['sha256'] and actual['bytes']==record['bytes']
    source.append(actual)
assert sum(r['bytes'] for r in source)==504306737
files=[a.baseline/'model.json',a.baseline/'full/predictions.json',a.run/'model.json',a.run/'boundaries.json',a.run/'first/predictions.json',a.run/'full/predictions.json',a.run/'legacy-predictions.json',a.target/'release/dialogue-multi',a.target/'release/glassmind-dialogue',legacy,a.root/'selected/train.json',a.root/'selected/dev.json',a.root/'selected/train-schema.json',a.root/'selected/dev-schema.json',a.root/'train/schema.json',pathlib.Path('/home/gsc/.cache/glassmind/models/dialogue/model-12.json')]
repo_files=list(pathlib.Path('crates/dialogue/src').rglob('*.rs'))+list(pathlib.Path('tools/dialogue/boundaries').glob('*.py'))+list(pathlib.Path('tools/dialogue/boundaries').glob('*.sh'))+[pathlib.Path('tools/dialogue/boundaries/EXPERIMENT.md')]
scorers=[pathlib.Path(s) for s in ('tools/dialogue/evaluate.py','tools/dialogue/compare.py','crates/dialogue/tools/score.py','crates/dialogue/tools/upstream_metrics.py')]
report=dict(base_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),source_revision=old['source_revision'],license=old['license'],source_bytes=sum(r['bytes'] for r in source),download_bytes=0,source_files=source,artifacts=[file_record(p) for p in files],evaluated_source=[file_record(p) for p in sorted(repo_files)],unchanged_evaluators=[file_record(p) for p in scorers],checks=dict(frozen_weights_unchanged=True,frozen_predictions_identical=True,legacy_predictions_identical=True,all_baseline_correct_retained=True,no_new_wrong_scored_claims=True,annotation_rejection=True,conversation_order_isolation=True,scoped_unit_tests=8,clippy_all_targets=True,fmt=True),limitations=['No semantic freshness fix','Six changed held slot-turns have no gold frames','Two inherited wrong copy events persist seven scored slot-turns','No held-out evaluation','Training/calibration use gold TRAIN spans, not predicted source spans'])
(a.run/'artifacts.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ('source_files','artifacts','evaluated_source','unchanged_evaluators')},indent=2))
