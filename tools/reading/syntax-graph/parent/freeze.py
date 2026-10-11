from pathlib import Path
import hashlib,itertools,json,random,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
items=[
('The sailor repaired the lantern.', [('sailor','repaired','nsubj'),('lantern','repaired','obj')]),
('A tired baker carried the basket.', [('baker','carried','nsubj'),('basket','carried','obj')]),
('Yesterday the mechanic inspected a truck.', [('mechanic','inspected','nsubj'),('truck','inspected','obj')]),
('The curious child carefully opened the parcel.', [('child','opened','nsubj'),('parcel','opened','obj')]),
('The musician near the fountain greeted Elena.', [('musician','greeted','nsubj'),('Elena','greeted','obj')]),
('After lunch, Rafael polished the mirror.', [('Rafael','polished','nsubj'),('mirror','polished','obj')]),
('The dog behind the fence chased a rabbit.', [('dog','chased','nsubj'),('rabbit','chased','obj')]),
('The doctor examined the patient yesterday.', [('doctor','examined','nsubj'),('patient','examined','obj')]),
('Did the pilot notice the storm?', [('pilot','notice','nsubj'),('storm','notice','obj')]),
('Which painting did Nora restore?', [('Nora','restore','nsubj'),('painting','restore','obj')]),
('Who repaired the bicycle?', [('Who','repaired','nsubj'),('bicycle','repaired','obj')]),
('Why did the farmer close the gate?', [('farmer','close','nsubj'),('gate','close','obj')]),
('The student did not understand the question.', [('student','understand','nsubj'),('question','understand','obj')]),
('The chef has prepared the soup.', [('chef','prepared','nsubj'),('soup','prepared','obj')]),
('The courier will deliver the package tomorrow.', [('courier','deliver','nsubj'),('package','deliver','obj')]),
('The hikers were watching the eagle.', [('hikers','watching','nsubj'),('eagle','watching','obj')]),
('The parcel was opened by Sofia.', [('parcel','opened','nsubj:pass'),('was','opened','aux:pass')]),
('The windows were cleaned by the caretaker.', [('windows','cleaned','nsubj:pass'),('were','cleaned','aux:pass')]),
('Élodie photographed Zürich.', [('Élodie','photographed','nsubj'),('Zürich','photographed','obj')]),
('Łukasz visited Málaga.', [('Łukasz','visited','nsubj'),('Málaga','visited','obj')]),
('The gardener watered roses and trimmed hedges.', [('gardener','watered','nsubj'),('roses','watered','obj'),('trimmed','watered','conj'),('hedges','trimmed','obj')]),
('The teacher praised the child who solved the puzzle.', [('teacher','praised','nsubj'),('child','praised','obj'),('who','solved','nsubj'),('puzzle','solved','obj')]),
('Although rain fell, the ranger checked the bridge.', [('ranger','checked','nsubj'),('bridge','checked','obj')]),
('The scientist quietly recorded the unexpected result.', [('scientist','recorded','nsubj'),('result','recorded','obj')]),
]
cases=[]
for i,(text,edges) in enumerate(items):
    forms=re.findall(r'\w+|[^\w\s]',text)
    def index(word):assert forms.count(word)==1;return forms.index(word)+1
    cases.append(dict(name=f'core_{i:02d}',text=text,forms=forms,edges=[dict(dependent=index(d),head=index(h),label=l) for d,h,l in edges]))
(HERE/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
for mode in ['forms','text']:(HERE/(mode+'.jsonl')).write_text(''.join(json.dumps({mode:c[mode]},ensure_ascii=False)+'\n' for c in cases))
# Independently enumerate all legal single-root trees for bounded arbitrary scores.
rng=random.Random(430019);graphs=[]
for n in [2,3,4,5]:
 for variant in range(3):
    scores=[[rng.randrange(-8,9) if variant else 0 for d in range(n+1)] for h in range(n+1)]
    best=None;optimal=0
    for heads in itertools.product(range(n+1),repeat=n):
        if heads.count(0)!=1 or any(h==d for d,h in enumerate(heads,1)):continue
        valid=True
        for d in range(1,n+1):
            seen=set();v=d
            while v:
                if v in seen:valid=False;break
                seen.add(v);v=heads[v-1]
            if not valid:break
        if not valid:continue
        score=sum(scores[h][d] for d,h in enumerate(heads,1))
        if best is None or score>best:best=score;optimal=1
        elif score==best:optimal+=1
    graphs.append(dict(nodes=n,scores=scores,maximum_single_root_score=best,optimal_trees=optimal))
(HERE/'decoder-cases.json').write_text(json.dumps(graphs,indent=2)+'\n')
(HERE/'PLAN.md').write_text('''# Frozen graph parser parent contracts
24 independently authored sentences/52 core labeled edges: modifiers, questions,
negation, auxiliaries, passives, coordination, relative clauses and Unicode names.
No old-correct core edge may be lost; full fixed corpus LAS>=19673 and UAS>=20540
remain mandatory independently. Forms/text inference must emit valid bounded
single-root trees and exact source spans. Semantic labels remain predictions.
Twelve small arbitrary score matrices have independently enumerated exact maximum
single-root tree scores, including negative scores and ties. Candidate decoder
must match the optimum with legal trees; do not require a particular tied tree.
Full DEV retained outputs will be independently rescored with unchanged audit,
including all2001 sentences/25148 words and68 duplicate sentences/137 words.
Legacy model/forms/raw behavior must remain byte-identical. No parent cases are
available to worker. Decoder adapter can change interface only after source review;
fixed expectations and acceptance gates cannot change.
''')
for mode in ['forms','text']:
    subprocess.run([str(ROOT/'graphsyntax-run/baseline/reading-syntax'),'predict','--model',str(ROOT/'graphsyntax-run/baseline/model.bin'),'--input',str(HERE/(mode+'.jsonl')),'--output',str(HERE/('baseline-'+mode+'.jsonl'))],check=True,timeout=30)
rows=[json.loads(s) for s in (HERE/'baseline-forms.jsonl').read_text().splitlines()]
correct=[]
for c,r in zip(cases,rows):
    assert r['forms']==c['forms'] and r['structurally_valid']
    correct.append([r['heads'][e['dependent']-1]==e['head'] and r['deprels'][e['dependent']-1]==e['label'] for e in c['edges']])
report=dict(cases=len(cases),edges=sum(map(len,correct)),right=sum(sum(x) for x in correct),complete=sum(all(x) for x in correct),correctness=correct)
(HERE/'baseline.json').write_text(json.dumps(report,indent=2)+'\n')
manifest={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in HERE.iterdir() if p.is_file() and p.name!='freeze.json'}
(HERE/'freeze.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='correctness'}))
