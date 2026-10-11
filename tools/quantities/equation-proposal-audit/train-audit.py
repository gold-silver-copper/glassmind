from pathlib import Path
import sys,json,hashlib,re,collections
root=Path('/home/gsc/.cache/glassmind/continuous-20261010');sys.path.insert(0,str(root/'quantities-run/tmp/quantities-deps'));import pyarrow.parquet as pq
source=root/'quantities-run/tmp/mathqa/train.parquet';rows=pq.read_table(source,columns=['Problem','linear_formula','category']).to_pylist();ops=collections.Counter();steps=collections.Counter();templates=collections.Counter();arith=0;bounded=0;constants=collections.Counter()
for row in rows:
 f=row['linear_formula'];names=re.findall(r'([a-z_]+)\(',f);ops.update(names);steps[len(names)]+=1
 if names and set(names)<={'add','subtract','multiply','divide'}:
  arith+=1
  if len(names)<=6:
   bounded+=1;templates[f]+=1;constants.update(re.findall(r'const_[a-z0-9_]+',f))
report={'input':str(source),'sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'bytes':source.stat().st_size,'rows':len(rows),'columns_read':['Problem','linear_formula','category'],'split':'official TRAIN only; no DEV program/answer labels inspected for this audit','operations':dict(ops.most_common()),'program_steps':dict(sorted(steps.items())),'four_rational_operations_rows':arith,'four_operations_at_most_six_steps':bounded,'distinct_bounded_templates':len(templates),'bounded_constants':dict(constants.most_common()),'baseline_quantities_binary_sha256':'5a94dba0b893fafed21906c2150e51de7fab94c67cadefba10008b2a187e4b9f','next_action':'Design a disjoint TRAIN fit/calibration experiment for learned bounded equation proposals with exact typed execution; compare against a frozen lexical retrieval baseline before DEV. Do not substitute model confidence for English entailment, or silently route unverified proposals into solve.'}
out=root/'coordinator/equation-proposal-audit/train-audit.json';out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ('operations','bounded_constants','program_steps')},indent=2));print('Top operations',ops.most_common(12));print('Constants',constants.most_common(15))
