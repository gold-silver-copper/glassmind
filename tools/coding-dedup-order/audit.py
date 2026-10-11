"""Audit owned evidence, unchanged frozen contracts, and old repair logic."""
import argparse,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);r=p.parse_args().run
for name in ['PLAN.md','cases.py','cases.json','supplement.json']:
 assert (r/'frozen-before'/name).read_bytes()==(Path('tools/coding-dedup-order')/name).read_bytes(),name
old=(r/'frozen-before/world-rust/src/checks.rs').read_text();new=Path('crates/world-rust/src/checks.rs').read_text()
def part(s,start,end):return s[s.index(start):s.index(end,s.index(start))]
for start,end in [('fn undo_legacy_slip(', '/// A scratch directory for trying changes'),('fn legacy_slips(', '/// Confirm a proposed witness'),('fn run_in_until(', '\nfn run_tools('),('fn cargo(', '\nfn text(')]:assert part(old,start,end)==part(new,start,end),start
native=json.loads((r/'native-evidence/contracts.json').read_text());assert len(native)==29
assert all(c['original_compiled'] and c['original_tests_failed'] for c in native)
assert sum(c['choice'] is not None for c in native)==13
amb=next(c for c in native if c['name']=='two_passing');assert amb['choice'] is None and amb['withdrawn'] and len(set(amb['native_outputs']))==2
choices=[]
for path in (r/'native-scratch').glob('*/choice.json'):
 c=json.loads(path.read_text())
 if path.parent.name.startswith('dedup-order-dedup-contract-two_passing-'):
  assert not c['accepted'] and not c['incomplete'] and len(c['passing'])==2
  assert len(c['native_checks'])==2 and all(x['result']=={'Ok':[]} for x in c['native_checks'])
  choices.append(str(path.relative_to(r)))
assert choices
for name in ['test-review.log','clippy-final.log','release-final.log']:
 text=(r/name).read_text();assert 'error:' not in text and 'FAILED' not in text,name
counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed', (r/'test-review.log').read_text())]
assert sum(counts)==54,counts
source=Path('/home/gsc/.cache/glassmind/continuous-20261010/extremum-source');m=json.loads((source/'manifest.json').read_text())
assert len(m['files'])==110 and sum(e['bytes'] for e in m['files'])==57265
for e in m['files']:assert hashlib.sha256((source/e['file']).read_bytes()).hexdigest()==e['sha256']
report=dict(native_contracts=29,positive=13,negative=15,ambiguous=1,all_originals_compiled_and_failed_executed_tests=True,world_rust_tests=sum(counts),unchanged_old_functions=['undo_legacy_slip','legacy_slips','run_in_until','cargo'],frozen_cases_unchanged=True,ambiguity_evidence=choices,readonly_TRAIN_manifest_verified=True,TRAIN_files=110,TRAIN_bytes=57265,limitations='Public tests do not prove intended semantics. Full765 is coordinator-only.')
(r/'audit.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
