"""Verify parent positive programs and binding boundaries with unchanged native code."""
from pathlib import Path
import hashlib,json,subprocess
base=Path(__file__).parent;work=base.parents[1];binary=work/'groundedsteps-source/equation_executor';cases=json.loads((base/'cases.json').read_text());assert hashlib.sha256((base/'cases.json').read_bytes()).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256']
def call(rows):
 r=subprocess.run([str(binary)],input=''.join(json.dumps(x)+'\n' for x in rows),text=True,capture_output=True,timeout=30);assert r.returncode==0,r.stderr;v=[json.loads(x) for x in r.stdout.splitlines()];assert len(v)==len(rows);return v
bound=call([dict(question=c['question'],program=None) for c in cases]);(base/'native-binding.jsonl').write_text(''.join(json.dumps(dict(name=c['name'],native=r),ensure_ascii=False)+'\n' for c,r in zip(cases,bound)))
arg=lambda kind,index:dict(kind=kind,index=index)
programs={
 'two_steps':{'steps':[dict(op='add',left=arg('input',0),right=arg('input',1)),dict(op='multiply',left=arg('step',0),right=arg('input',2))]},
 'ordered_division':{'steps':[dict(op='subtract',left=arg('input',0),right=arg('input',1)),dict(op='divide',left=arg('step',0),right=arg('input',2))]},
}
programs['unicode_spans']=programs['two_steps'];records=[]
for c in cases:
 if not c['unique_program_expected']:continue
 p=programs[c['name']];v=call([dict(question=c['question'],program=p)])[0];assert v['status']=='proposed' and v['value']==('2' if c['name']=='ordered_division' else '135')
 for number in v['numbers']:assert c['question'].encode()[number['start']:number['end']].decode()==number['literal']
 records.append(dict(name=c['name'],program=p,native=v))
byname={c['name']:v for c,v in zip(cases,bound)};assert byname['number_words']['status']=='refusal' and byname['unicode_number']['status']=='refusal';assert [n['value'] for n in byname['duplicate_occurrences']['numbers']]==['7','7'];assert [n['value'] for n in byname['input_prior_collision']['numbers']]==['7','8','9','15'];assert [n['value'] for n in byname['constant_collision']['numbers']]==['8','12']
report={'scope':'Gold provided parent programs and original numeric binding, not free solver accuracy','question_binding_cases':len(cases),'positive_exact_native_executions':records,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()};(base/'native-baseline.json').write_text(json.dumps(report,indent=2)+'\n');print('19 bindings and three native positive executions verified')
