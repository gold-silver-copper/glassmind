import ast,hashlib,json,os,shutil,sys,collections
from pathlib import Path
import pyarrow.parquet as pq
ROOT=Path.cwd(); S=ROOT.parent/'groundingaudit-source'; R=Path(os.environ.get('EQUIV_RUN',str(ROOT.parent/'equivbinding-run/experiment'))); O=ROOT/'tools/quantities/equivalent-binding'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def dump(p,x):p.write_text(json.dumps(x,sort_keys=True,indent=2)+'\n')
def jl(p,x):p.write_text(''.join(json.dumps(v,separators=(',',':'))+'\n' for v in x))
R.mkdir(parents=True,exist_ok=True)
assert not (R/'corpus-frozen.json').exists(), 'use a fresh AUDIT_RUN; preserve previous attempts'
manifest=json.loads((S/'manifest.json').read_text())
for f in manifest['files']:assert sha(S/f['file'])==f['sha256'] and (S/f['file']).stat().st_size==f['bytes'],f
runtime={}
dump(R/'runtime-before.json',runtime)
shutil.copy2(ROOT/'tools/quantities/typed-label-metric/metric.py',R/'metric.py');sys.path.insert(0,str(R))
from metric import parse_gold,compare
# Import only these exact original parser definitions, never its main or data access.
tree=ast.parse((S/'old-parser.py').read_text()); selected=[]
for n in tree.body:
 if isinstance(n,ast.FunctionDef) and n.name in ('program','group','grade'):selected.append(n)
 if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='CONST' for t in n.targets):selected.append(n)
import re
exec(compile(ast.Module(body=selected,type_ignores=[]),str(S/'old-parser.py'),'exec'))
rows=pq.read_table(S/'train.parquet',columns=['Problem','linear_formula','options','correct']).to_pylist();assert len(rows)==29837
ps=[program(r['linear_formula']) for r in rows]; groups=[group(r['Problem']) for r in rows]
fit=[i for i,g in enumerate(groups) if int(g,16)%5];cal=[i for i,g in enumerate(groups) if not int(g,16)%5]
assert {'fit':fit,'calibration':cal,'group_hashes':groups}==json.loads((S/'split.json').read_text())
assert not {groups[i] for i in fit}&{groups[i] for i in cal}
ex=[json.loads(l) for l in (S/'train-executions.jsonl').read_text().splitlines()];assert len(ex)==len(rows)
calq=[json.loads(l) for l in (S/'calibration-questions.jsonl').read_text().splitlines()];assert calq==[rows[i]['Problem'] for i in cal]
clean={}; exclusions={}; membership=[]
for tag,ids in [('FIT',fit),('CAL',cal)]:
 clean[tag]=[]; exclusions[tag]=collections.Counter()
 for i in ids:
  status='unsupported_annotation' if ps[i] is None else 'binding_or_execution_refusal' if ex[i]['status']!='proposed' else 'label_'+grade(rows[i],ex[i])
  if status=='label_correct':clean[tag].append(i)
  else:exclusions[tag][status]+=1
  membership.append(dict(index=i,split=tag,group=groups[i],clean=status=='label_correct',status=status))
expected=[dict(question=rows[i]['Problem'],program=ps[i]) for i in clean['FIT']]
assert expected==[json.loads(l) for l in (S/'fit-input.jsonl').read_text().splitlines()];assert len(expected)==4590;assert len(clean['CAL'])==1182
jl(R/'membership.jsonl',sorted(membership,key=lambda x:x['index']))
jl(R/'clean.jsonl',[dict(index=i,split=tag,question=rows[i]['Problem'],program=ps[i],options=rows[i]['options'],correct=rows[i]['correct']) for tag in ['FIT','CAL'] for i in clean[tag]])
jl(R/'gold-all-input.jsonl',[dict(question=r['Problem'],program=p) for r,p in zip(rows,ps)])
jl(R/'replay200-input.jsonl',calq[:200]);jl(R/'replay200-expected.jsonl',[json.loads(l) for l in (S/'calibration-candidate.jsonl').read_text().splitlines()][:200])
dump(R/'corpus-frozen.json',dict(total=len(rows),fit=len(fit),cal=len(cal),fit_groups=len({groups[i] for i in fit}),cal_groups=len({groups[i] for i in cal}),clean={k:len(v) for k,v in clean.items()},exclusions=exclusions,structures_fit=len({json.dumps(ps[i],sort_keys=True) for i in clean['FIT']}),files={p.name:sha(p) for p in [O/'PLAN.md',R/'membership.jsonl',R/'clean.jsonl',S/'split.json',S/'train.parquet',S/'model.json',R/'metric.py']}))
print((R/'corpus-frozen.json').read_text())

shutil.copy2(S/'fit-input.jsonl',R/'fit-input.jsonl')
