"""Question-only parent expectations; source labels are checked only after freeze."""
from pathlib import Path
import hashlib,json,subprocess
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/quantity-graphs')
rows=json.loads((r/'source/parent-question-only.json').read_text());assert all('answer' not in x and 'final_answer' not in x for x in rows)
ref=lambda s:dict(kind={'i':'input','s':'step','c':'constant'}[s[0]],index=int(s[1:]))
def graph(*triples):return dict(steps=[dict(op=op,left=ref(a),right=ref(b)) for op,a,b in triples])
expected={
5799:dict(expected='22',program=graph(('add','i0','i1'),('multiply','s0','c2')),why='Outward walking plus bus distance, doubled because he comes home the same way. Two directions license constant2.',mutations=[dict(changes={0:'5'},expected='24'),dict(changes={1:'9'},expected='26')]),
312:dict(expected='20',program=graph(('add','c1','c2'),('divide','i0','s0')),why='Ordinary emu anatomy gives one head and two legs per bird; explicitly compute three counted parts per emu, then divide total parts by this count.',assumptions=['Ordinary uninjured emus have one head and two legs. This is generic anatomical knowledge, not an input-dependent hidden subtotal.'],mutations=[dict(changes={0:'63'},expected='21'),dict(changes={0:'90'},expected='30')]),
5151:dict(expected='85',program=graph(('add','i1','i2'),('subtract','i0','i1'),('add','s1','s0')),why='Incoming cars are departed cars plus the excess5. Final cars are initial minus departed plus incoming. A simpler initial-plus-excess graph is equally acceptable if fully grounded; departures cancel rather than becoming an independent final effect.',mutations=[dict(changes={2:'7'},expected='87'),dict(changes={1:'16'},expected='85'),dict(changes={0:'70'},expected='75')]),
7044:dict(reject=True,allowed_statuses=['ambiguous','unsupported','label_error'],why='Four dollars for the next three knives can denote a group price or a per-knife tier price; the wording does not explicitly say each in that tier. For nine knives the two readings give24 or32. A reference rationale cannot choose a missing price-unit premise without acknowledging it.'),
2791:dict(expected='155',program=graph(('multiply','i1','i3'),('multiply','i2','i4'),('add','s0','s1')),why='Separate ticket spending is adult price times adult count plus child price times child count. Family-pass price120 is a distractor, even though it equals the original computed child subtotal.',mutations=[dict(changes={4:'5'},expected='135'),dict(changes={3:'2'},expected='190'),dict(changes={0:'999'},expected='155')]),
5197:dict(reject=True,allowed_statuses=['ambiguous','unsupported','label_error'],why='Mother rode10timesmore than John can mean10additional rides in this count context, or a multiplicative comparison. Those readings yield different totals (95 for ten additional rides versus391 for ten times as many; a strict ten-times-more interpretation differs again). The question-only parent reference requires acknowledging this ambiguous comparison rather than guessing the source rationale.'),
3325:dict(reject=True,allowed_statuses=['ambiguous','unsupported','label_error'],why='The family stores bicycles in the garage when they are not being used, but the question does not specify how many of its four bicycles are currently there rather than in use. Eighty is the total spoke count under an all-stored assumption; current inside-garage count needs that contingent premise. This is a conservative location-interpretation refusal, not an arithmetic contradiction.'),
1626:dict(expected='45',program=graph(('multiply','i0','i1'),('multiply','i2','i3'),('add','s0','s1')),why='Each bill count multiplies its denomination; both product totals are added. The one ten-dollar bill remains an input count rather than an assumed constant.',mutations=[dict(changes={0:'8'},expected='50'),dict(changes={2:'2'},expected='55')])}
assert list(expected)==[x['row_id'] for x in rows]
requests=[];checks=[];cases=[]
for row in rows:
 d=expected[row['row_id']];cases.append(dict(id=row['id'],question_sha256=hashlib.sha256(row['question'].encode()).hexdigest(),**d))
 if d.get('reject'):continue
 requests.append(dict(question=row['question'],program=d['program']));checks.append(dict(id=row['id'],kind='original',expected=d['expected']))
 for change in d['mutations']:
  q=row['question'].encode()
  for index,value in sorted(change['changes'].items(),key=lambda x:row['numbers'][x[0]]['start'],reverse=True):
   n=row['numbers'][index];q=q[:n['start']]+value.encode()+q[n['end']:]
  requests.append(dict(question=q.decode(),program=d['program']));checks.append(dict(id=row['id'],kind='question_only_counterfactual',**change))
executor=r.parent.parent/'groundedsteps-source/equation_executor'
p=subprocess.run([str(executor)],input=''.join(json.dumps(x)+'\n' for x in requests),text=True,capture_output=True,check=True,timeout=30)
outputs=[json.loads(x) for x in p.stdout.splitlines()];assert len(outputs)==len(checks)
for c,o in zip(checks,outputs,strict=True):assert o['status']=='proposed' and o['value']==c['expected'],(c,o)
out=r/'parent';out.mkdir(exist_ok=False)
for name,data in [('expectations.json',cases),('native-checks.json',dict(checks=checks,outputs=outputs))]:
 p=out/name;p.write_text(json.dumps(data,indent=2)+'\n');p.chmod(0o444)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
freeze=dict(question_only_before_source_labels=True,teacher_calls=0,parent_cases=8,required_nonacceptance=[7044,5197,3325],native_requests=len(requests),all_native_manual_expectations_match=True,parent_question_only_sha256=sha(r/'source/parent-question-only.json'),files={p.name:sha(p) for p in out.iterdir()})
(out/'freeze.json').write_text(json.dumps(freeze,indent=2)+'\n');(out/'freeze.json').chmod(0o444)
# Only now inspect the reference labels, without editing the frozen expectations.
batch={x['id']:x for x in json.loads((r/'source/batch.json').read_text())['rows']}
comparison=[dict(id=c['id'],parent_requires_refusal=c.get('reject',False),source_label=batch[c['id']]['final_answer'],parent_value=c.get('expected'),agrees_if_accepted=c.get('reject',False) or c['expected']==batch[c['id']]['final_answer']) for c in cases]
(out/'source-label-comparison.json').write_text(json.dumps(comparison,indent=2)+'\n')
print(json.dumps(dict(freeze=freeze,label_comparison=comparison)))
