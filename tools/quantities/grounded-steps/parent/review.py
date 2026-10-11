"""Independent source/membership/native replay and frozen parent converter contracts."""
from pathlib import Path
from fractions import Fraction
from collections import Counter
import ast,hashlib,importlib.util,json,re,subprocess
base=Path(__file__).parent;work=base.parents[1];source=work/'groundedsteps-source';frozen=work/'groundedsteps-run/frozen-candidate';out=base/'candidate';out.mkdir();sha=lambda b:hashlib.sha256(b).hexdigest();read=lambda p:[json.loads(line) for line in p.open()]
manifest=json.loads((frozen/'artifact-manifest.json').read_text())['files']
for r in manifest:
 b=(frozen/r['file']).read_bytes();assert len(b)==r['bytes'] and sha(b)==r['sha256']
full=frozen/'full';exe=source/'equation_executor'
def native(rows,name):
 raw=''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in rows).encode();(out/(name+'-requests.jsonl')).write_bytes(raw);p=subprocess.run([str(exe)],input=raw,capture_output=True,timeout=60);assert p.returncode==0,p.stderr;(out/(name+'-outputs.jsonl')).write_bytes(p.stdout);return [json.loads(x) for x in p.stdout.splitlines()]
spec=importlib.util.spec_from_file_location('submitted_converter',frozen/'audit.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
cases=json.loads((base/'cases.json').read_text());assert sha((base/'cases.json').read_bytes())==json.loads((base/'freeze.json').read_text())['sha256'];binding=native([dict(question=c['question'],program=None) for c in cases],'parent-binding');contracts=[]
expected={r['name']:r for r in json.loads((base/'native-baseline.json').read_text())['positive_exact_native_executions']}
for c,b in zip(cases,binding):
 result=module.convert(c,b);actual=result['uniquely_grounded_typed_program'];assert actual==c['unique_program_expected'],(c['name'],result)
 if actual:
  assert result['program']==expected[c['name']]['program'];v=native([dict(question=c['question'],program=result['program'])],c['name'])[0];assert v==expected[c['name']]['native']
 contracts.append(dict(name=c['name'],expected=c['unique_program_expected'],actual=actual,reasons=result['reasons']))
(out/'contracts.json').write_text(json.dumps(contracts,indent=2)+'\n')
raw=(source/'train.jsonl').read_bytes().splitlines(keepends=True);rows=[json.loads(x) for x in raw];records=read(full/'records.jsonl');members=read(full/'membership.jsonl');original_bindings=read(full/'question-only-binding.outputs.jsonl');assert len(rows)==len(records)==len(members)==7473
requests=[dict(question=r['question'],program=None) for r in rows];assert requests==read(full/'question-only-binding.requests.jsonl');bindings=native(requests,'all-binding');assert bindings==original_bindings
constants=list(map(Fraction,['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']));accepted=[];groups={};leaf_count=0;expressions=0
# Different parser from the submitted recursive descent; never evaluates Python code.
def arithmetic(text):
 text=text.strip();tree=ast.parse(text,mode='eval')
 def visit(n):
  if isinstance(n,ast.Constant) and type(n.value) in [int,float]:return Fraction(ast.get_source_segment(text,n))
  assert isinstance(n,ast.BinOp) and type(n.op) in [ast.Add,ast.Sub,ast.Mult,ast.Div],ast.dump(n)
  a,b=visit(n.left),visit(n.right)
  if isinstance(n.op,ast.Add):return a+b
  if isinstance(n.op,ast.Sub):return a-b
  if isinstance(n.op,ast.Mult):return a*b
  return a/b
 return visit(tree.body)
for i,(row,record,member,b) in enumerate(zip(rows,records,members,bindings)):
 normalized=re.sub('[0-9]+','#',' '.join(row['question'].lower().split()));group=sha(normalized.encode());subset='TRAIN-CAL' if int(group,16)%5==0 else 'TRAIN-FIT';assert member==dict(row_id=i,raw_line_sha256=sha(raw[i]),question_sha256=sha(row['question'].encode()),answer_sha256=sha(row['answer'].encode()),normalized_question=normalized,group_sha256=group,subset=subset)
 assert record['row_id']==i and record['subset']==subset and record['group_sha256']==group
 assert groups.setdefault(group,subset)==subset
 if b['status']=='bound':
  for n in b['numbers']:assert row['question'].encode()[n['start']:n['end']].decode()==n['literal']
 if record['annotation_parses']:
  assert len(record['annotations'])==len(re.findall(r'<<([^<>]*)>>',row['answer']))
  for ann in record['annotations']:
   v=arithmetic(ann['expression']);assert v==Fraction(ann['computed'])==Fraction(ann['label'].replace(',',''));expressions+=1
  final=row['answer'].split('####')[1].strip().replace(',','');assert record['final_consistent']==(v==Fraction(final))
 if 'leaves' in record:
  inputs=[Fraction(n['value']) for n in b['numbers']];prior=[Fraction(n['value']) for n in record['expression_nodes']];product=1
  for leaf in record['leaves']:
   wanted=[];value=Fraction(leaf['value'])
   for kind,values in [('input',inputs),('step',prior[:leaf['available_before']]),('constant',constants)]:wanted.extend(dict(kind=kind,index=j) for j,x in enumerate(values) if x==value)
   assert wanted==leaf['alternatives'];product*=len(wanted);leaf_count+=1
  assert str(product)==record['candidate_program_count']
 if record['accepted']:accepted.append((i,record))
assert len(accepted)==354
execution=native([dict(question=rows[i]['question'],program=r['program']) for i,r in accepted],'all-gold-execution');assert execution==read(full/'gold-provided-execution.outputs.jsonl')
assert [dict(question=rows[i]['question'],program=r['program']) for i,r in accepted]==read(full/'gold-provided-execution.requests.jsonl')
for (i,r),v in zip(accepted,execution):assert v==r['native_gold_execution'] and v['status']=='proposed' and Fraction(v['value'])==Fraction(r['final_answer']) and v['numbers']==bindings[i]['numbers']
reported=json.loads((full/'report.json').read_text());counts={k:sum(r[k] is True for r in records) for k in ['annotation_present','annotation_parses','arithmetic_valid','final_consistent','exact_native_binder_accepted','uniquely_grounded_typed_program','ambiguity','accepted']}
for k,n in counts.items():assert n==reported['all']['stages'][k]['true']
reasons=dict(Counter(x for r in records for x in r['reasons']));assert reasons==reported['all']['exclusion_reasons'];assert not reported['gate']['coverage_pass'];assert read(frozen/'sample/records.jsonl')==records[:200]
report=dict(manifest_entries_verified=len(manifest),parent_contracts=len(contracts),all_parent_contracts_passed=True,train_rows=len(rows),groups=len(groups),subsets=dict(Counter(m['subset'] for m in members)),question_only_native_bindings_exact=len(bindings),gold_native_executions_exact=len(execution),independent_arithmetic_expressions=expressions,independent_leaf_candidate_sets=leaf_count,stages=counts,exclusions=reasons,accepted_by_subset=dict(Counter(r['subset'] for _,r in accepted)),gate_required=1000,gate_actual=len(accepted),decision='rejected',scope='Converted gold TRAIN labels, not predictions or semantic proof');(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
