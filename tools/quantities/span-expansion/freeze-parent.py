"""Prospective root-authored source references, hidden from the four teachers."""
from pathlib import Path
import hashlib,json,subprocess
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/span-expansion')
batch=json.loads((r/'source/batch.json').read_text())['rows']
rows={batch[i]['row_id']:batch[i] for i in (0,1,32,33,64,65,96,97)}
expected={
6098:dict(assignments=['i2','i0','s0','i1','i2','s1'],why='Elapsed minutes are Matt steps divided by Matt speed. Computed minutes multiply the extra speed, then extra steps add to Matt steps. The input 5 is an extra rate, not a constant.',mutations=[(1,'10','330'),(0,'25','264')]),
1490:dict(assignments=['i1','i3','i2','i0','s1'],why='Apple and grape weights are separate occurrences of 3, each used once. Their direct swap is equivalent. Add orange weight, then subtract computed combined weight from total fruit weight.',mutations=[(1,'4','2'),(3,'5','1')]),
818:dict(assignments=['i0','i1','i2','i3','s1','s2','c11'],why='Sum all three stated task counts, multiply computed total by stated minutes per task and divide computed minutes by 60 minutes/hour. Each input count/rate is variable; 12 is a computed total, not a constant.',assumptions=['Everything on the list takes ten minutes is read distributively per listed task, as supported by the counting request.'],mutations=[(1,'2','13/6'),(3,'15','3')]),
3095:dict(assignments=['i0','c2','c12','i1','s0','i2','c12','s2'],why='Twice licenses 2 multiplying giraffe count. Percentage conversion licenses 100 divided by penguin percent, then multiply computed penguin count. Elephant percent over 100 multiplies computed zoo total.',mutations=[(0,'10','4'),(1,'10','4'),(2,'6','3')]),
4498:dict(reject=True,allowed_statuses=['label_error','ambiguous','unsupported'],why='The cut board is stated to be longer than the other boards, yet the rationale adds 7 to infer those other boards were longer. Cutting 7 from each cannot equalize unequal boards. If the board alone were cut, the inferred shorter boards would be 111, not 125. The question has conflicting premises and two coreferential numeric mentions; an accepted 118+7 graph would repair the question silently.'),
3040:dict(assignments=['i0','c3','s0','i1','s1','i0'],why='Thrice licenses 3 times Tim age for Rommel. Jenny is the computed Rommel age plus the stated gap. Subtract original Tim age from computed Jenny age.',mutations=[(0,'6','14'),(1,'3','13')]),
5266:dict(assignments=['i3','i0','i4','i1','s0','s1','s2','i2'],why='Lawn count times lawn rate yields revenue. Video-game count uses its own 5 occurrence times game price; subtract computed game cost. Remaining money divides by the separate book-price input 5.',assumptions=['Given original and selected mutations have integral affordable book counts; arbitrary nonintegral cases would require floor division absent from this graph.'],mutations=[(2,'10','30'),(4,'4','69'),(3,'36','63')]),
917:dict(assignments=['i0','i2','i1','i2','s1','s0'],why='Both climbers use their own speed times the shared duration. Computed Jason height minus computed Matt height gives the difference. Speeds 6 and 12 are inputs, not constants.',mutations=[(0,'5','49'),(1,'15','63'),(2,'8','48')])}
assert list(expected)==list(rows)
ref=lambda text:dict(kind={'i':'input','s':'step','c':'constant'}[text[0]],index=int(text[1:]))
requests=[];checks=[];cases=[]
for i,d in expected.items():
 row=rows[i];case=dict(id=row['id'],row_id=i,question_sha256=hashlib.sha256(row['question'].encode()).hexdigest(),**d)
 if not d.get('reject'):
  a=list(map(ref,d['assignments']));steps=[]
  for node in row['expression_nodes']:
   resolve=lambda side:a[side['leaf']] if 'leaf' in side else side
   steps.append(dict(op=node['op'],left=resolve(node['left']),right=resolve(node['right'])))
  program=dict(steps=steps);case['program']=program
  requests.append(dict(question=row['question'],program=program));checks.append(dict(id=row['id'],kind='original',expected=row['final_answer']))
  for index,replacement,value in d['mutations']:
   n=row['numbers'][index];q=row['question'].encode();changed=(q[:n['start']]+replacement.encode()+q[n['end']:]).decode()
   requests.append(dict(question=changed,program=program));checks.append(dict(id=row['id'],kind='source_counterfactual',input=index,replacement=replacement,expected=value))
 cases.append(case)
executor=r.parent.parent/'groundedsteps-source/equation_executor'
p=subprocess.run([str(executor)],input=''.join(json.dumps(x)+'\n' for x in requests),text=True,capture_output=True,check=True,timeout=30)
outputs=[json.loads(x) for x in p.stdout.splitlines()];assert len(outputs)==len(checks)
for c,o in zip(checks,outputs,strict=True):assert o['status']=='proposed' and o['value']==c['expected'],(c,o)
out=r/'parent';out.mkdir(exist_ok=False)
for name,data in [('expectations.json',cases),('native-checks.json',dict(checks=checks,outputs=outputs))]:
 p=out/name;p.write_text(json.dumps(data,indent=2)+'\n');p.chmod(0o444)
report=dict(frozen_before_teacher=True,teacher_calls=0,parent_cases=8,required_nonacceptance=[4498],native_requests=len(requests),all_native_expectations_match=True,batch_sha256=hashlib.sha256((r/'source/batch.json').read_bytes()).hexdigest(),files={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.iterdir()})
(out/'freeze.json').write_text(json.dumps(report,indent=2)+'\n');(out/'freeze.json').chmod(0o444)
print(json.dumps(report))
