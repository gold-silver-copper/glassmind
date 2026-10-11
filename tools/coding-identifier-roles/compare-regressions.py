import hashlib,json,os
from pathlib import Path
run=Path(os.environ['TMPDIR']).parent
rows=[]
for name,keys in [('coding-metatypes-dev',['total','passed','usable','safe']),('coding-answerfiles-dev',['total','usable','safe']),('coding-answerfiles-regression',['total','right'])]:
 b=json.loads((run/'regressions-baseline'/name/'results.json').read_text());c=json.loads((run/'regressions-candidate'/name/'results.json').read_text())
 before={k:b[k] for k in keys};after={k:c[k] for k in keys};assert before==after
 # Compare every predicate/result, excluding path-dependent native diagnostics.
 br=b['rows'];cr=c['rows'];assert len(br)==len(cr)
 for old,new in zip(br,cr):
  for key in ['case','prompt','question','passed','expected','answer','usable','safe','changed','added','right','returncode']:
   if key in old and key in new:
    left,right=old[key],new[key]
    if isinstance(left,str) and isinstance(right,str):
     left=left.replace(str(run/'regressions-baseline'),'<RUN>')
     right=right.replace(str(run/'regressions-candidate'),'<RUN>')
    assert left==right,(name,key,old,new)
 rows.append(dict(adapter=name,baseline=before,candidate=after,all_row_predicates_equal=True))
b=json.loads((run/'regressions-baseline/coding-edits-dev/results.json').read_text());c=json.loads((run/'regressions-candidate/coding-edits-dev/results.json').read_text())
assert [(x['case'],x['pass'],x['changed'],x['exact_source'],x['behavior']) for x in b]==[(x['case'],x['pass'],x['changed'],x['exact_source'],x['behavior']) for x in c]
rows.append(dict(adapter='coding-edits-dev',baseline_pass=sum(x['pass'] for x in b),candidate_pass=sum(x['pass'] for x in c),total=len(b),all_row_predicates_equal=True))
assert (run/'addfield-baseline/status.txt').read_text()==(run/'addfield-candidate/status.txt').read_text()
for line in (run/'addfield-baseline/status.txt').read_text().splitlines():
 name,code=line.split()
 if code=='0':
  b=json.loads((run/'addfield-baseline'/name/'results.json').read_text());c=json.loads((run/'addfield-candidate'/name/'results.json').read_text())
  assert [(x['changed'],x['passed']) for x in b]==[(x['changed'],x['passed']) for x in c]
 else:
  b=run/'addfield-baseline'/name/name;c=run/'addfield-candidate'/name/name
  for file in ['src/lib.rs','src/main.rs','Cargo.toml']:
   assert (b/file).read_bytes()==(c/file).read_bytes()
rows.append(dict(adapter='coding-addfields-dev',baseline_pass=14,candidate_pass=14,total=15,inherited_failure='live-rollback: both binaries edit; original adapter TypeError on expected=None; exact final source identical'))
Path('tools/coding-identifier-roles/reports/regression-comparison.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2))
