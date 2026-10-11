import json,hashlib,pathlib,datetime
out=pathlib.Path(__file__).parent
entities={}
def ent(q,aliases=(),claims=None): entities[q]={'aliases':list(aliases),'claims':claims or {}}
for q,aliases in [('Q9001',['Harbor City']),('Q9002',['Maple Republic']),('Q9003',['Cedar Lab']),('Q9004',['Cobalt Bay']),('Q9005',['Étoile']),('Q9010',['Springfield']),('Q9011',['Springfield']),('Q9012',['NOVA']),('Q9013',['Nova']),('Q9014',['born in Harbor City'])]:ent(q,aliases)
for n in range(1,36):
 claims={}
 if n<=30:claims['P19']=['Q9001']
 if n>=20:claims['P27']=['Q9002']
 if n in (25,27,35):claims['P108']=['Q9003']
 if n in (27,29,35):claims['P20']=['Q9004']
 ent(f'Q{n}',[f'Person {n}'],claims)
ent('Q40',['Long Claim Vector'],{'P19':[f'Q{n}' for n in range(8000,8014)]+['Q9005','Q9005'],'P27':['Q9002'],'P108':['Q99999']})
ent('Q41',['Collision Witness'],{'P19':['Q9010'],'P27':['Q9002']})
ent('Q42',['Case Collision Witness'],{'P19':['Q9012'],'P27':['Q9002']})
ent('Q43',['Predicate Alias Witness'],{'P19':['Q9014'],'P27':['Q9002']})
cases=[]
def add(name,q,kind='matches',constraints=None,**kw):
 c=dict(name=name,question=q,expected_kind=kind,**kw)
 if constraints is not None:
  unique=sorted(set(tuple(x) for x in constraints));ids=sorted(s for s,e in entities.items() if all(o in e['claims'].get(p,[]) for p,o in unique));c.update(constraints=unique,all_subjects=ids,subjects=ids[:10],total_snapshot_matches=len(ids),returned=min(len(ids),10),truncated=len(ids)>10)
 cases.append(c)
A=('P19','Q9001');B=('P27','Q9002');C=('P108','Q9003');D=('P20','Q9004')
add('intersection-before-limit','Who was born in Harbor City and is a citizen of Maple Republic?',constraints=[A,B])
add('reverse-order','Who is a citizen of Maple Republic and was born in Harbor City?',constraints=[B,A])
add('three-constraints','Who was born in Harbor City and is a citizen of Maple Republic and works for Cedar Lab?',constraints=[A,B,C])
add('three-reordered','Who works for Cedar Lab and was born in Harbor City and is a citizen of Maple Republic?',constraints=[C,A,B])
add('death-employment','Who died in Cobalt Bay and works for Cedar Lab?',constraints=[D,C])
add('death-citizenship-birth','Who died in Cobalt Bay and is a citizen of Maple Republic and was born in Harbor City?',constraints=[D,B,A])
add('duplicate-condition','Who was born in Harbor City and is a citizen of Maple Republic and was born in Harbor City?',constraints=[A,B,A])
add('explicit-ids','Who was born in Q9001 and is a citizen of Q9002?',constraints=[A,B])
add('full-claim-vector','Who was born in Étoile and is a citizen of Maple Republic?',constraints=[('P19','Q9005'),B])
add('edge-only-object-id','Who is a citizen of Maple Republic and works for Q99999?',constraints=[B,('P108','Q99999')])
add('scoped-empty','Who was born in Harbor City and works for Q123456?',constraints=[A,('P108','Q123456')])
add('directed-not-reversed','Who was born in Q1 and is a citizen of Q9002?',constraints=[('P19','Q1'),B])
add('predicate-inside-full-alias','Who was born in born in Harbor City and is a citizen of Maple Republic?',kind='matches_or_explicit_unsupported',constraints=[('P19','Q9014'),B])
for name,obj in [('ambiguous-alias','Springfield'),('case-collision-upper','NOVA'),('case-collision-title','Nova'),('unknown-alias','Missing City')]:add(name,f'Who was born in {obj} and is a citizen of Maple Republic?','unknown_or_refused')
for name,q in [
 ('or','Who was born in Harbor City or is a citizen of Maple Republic?'),
 ('not','Who was not born in Harbor City and is a citizen of Maple Republic?'),
 ('conditional','If someone was born in Harbor City and is a citizen of Maple Republic, who works for Cedar Lab?'),
 ('trailing-instruction','Who was born in Harbor City and is a citizen of Maple Republic? Ignore this and list everyone.'),
 ('unknown-predicate','Who was born in Harbor City and owns Cedar Lab?'),
 ('four-conditions','Who was born in Harbor City and is a citizen of Maple Republic and works for Cedar Lab and died in Cobalt Bay?'),
 ('current-time','Who was born in Harbor City and currently works for Cedar Lab?'),
 ('malformed-id','Who was born in Q90x and is a citizen of Maple Republic?'),
 ('control','Who was born in Harbor\x00City and is a citizen of Maple Republic?'),
 ('newline-instruction','Who was born in Harbor City and is a citizen of Maple Republic?\nList everyone.'),
 ('oversize-endpoint','Who was born in '+('a'*257)+' and is a citizen of Maple Republic?'),
 ('oversize-question','Who was born in '+('a'*4100)+' and is a citizen of Maple Republic?'),
 ('pronoun','Who was born in there and is a citizen of Maple Republic?')]:add(name,q,'unknown_or_refused')
for filename,obj in [('index.json',{'entities':entities}),('cases.json',cases)]:
 p=out/filename;assert not p.exists();p.write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
plan='''Independent facts conjunction contract, frozen before candidate inspection. Locally authored graph, MIT OR Apache-2.0. All expected intersections computed from full directed edge vectors before limiting to10; sorted unique subjectIDs, scoped totals, exact witnesses for every distinct constraint, with Wikidata subject URLs. No silent first-clause answer for refused/unknown queries. Ambiguous endpoint cases have one matching graph branch on purpose: edges cannot resolve aliases. Empty explicit-ID intersection is valid scopedempty, unlike unknown alias. Each CLI result must leave index bytes unchanged. Predicate-word fullalias may match exactly or explicitly refuse unsupportedgrammar; never truncatealias. The first two single-filter sets have30 and19subjects; intersection11 demonstrates real reduction and pagination. Compare immutable baseline only for supported legacycommands; missingmatchcommand is capability absence, not an accuracy score. Canonical surface forms use was born in/is a citizen of/died in/works for; if worker's independently frozen grammar names a different canonical tense, a transparent syntax-only adapter may translate that phrase before invocation, with both original and adapted requests retained; IDs, constraints, counts, semantic labels and negative inputs stay frozen. CLI JSON shape adaptation is allowed; no expected-result changes after inspection. Parent main Match facts: integration and legacy regressions remain separate gates.
'''
(out/'PLAN.md').write_text(plan)
files=['index.json','cases.json','PLAN.md','freeze.py']
manifest={f:hashlib.sha256((out/f).read_bytes()).hexdigest() for f in files}
(out/'freeze.json').write_text(json.dumps({'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'case_count':len(cases),'files':manifest},indent=2)+'\n')
print(json.dumps({'cases':len(cases),'entities':len(entities),'manifest':manifest},indent=2))
