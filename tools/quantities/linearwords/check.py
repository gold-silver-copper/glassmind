#!/usr/bin/env python3
"""Local development cross-checks; no evaluation definitions are changed."""
import hashlib, json, os, subprocess
from fractions import Fraction as F
from pathlib import Path
from itertools import permutations

TMP = Path(os.environ['TMPDIR'])
BINARY = Path(os.environ.get('CANDIDATE_BINARY', str(Path(os.environ['CARGO_TARGET_DIR'])/'debug/glassmind-quantities')))
def run(binary, questions):
    result = subprocess.run([str(binary),'--jsonl'],input=''.join(json.dumps(q)+'\n' for q in questions),text=True,capture_output=True,check=True,timeout=120)
    answers = [json.loads(x) for x in result.stdout.splitlines()]
    assert len(answers)==len(questions)
    return answers

def canonical(a):
    return [a['status'],a.get('value'),a.get('unit')]

def main():
    rows=[]
    for name in ['synthetic-dev.jsonl','extended-dev.jsonl']:
        rows.extend(json.loads(s) for s in (Path('crates/quantities/dev')/name).read_text().splitlines())
    questions=[r['question'] for r in rows]
    before=run(TMP/'glassmind-quantities-baseline', questions)
    after=run(BINARY,questions)
    changes=[{'id':r['id'],'before':a,'after':b} for r,a,b in zip(rows,before,after) if canonical(a)!=canonical(b)]
    counts={'right':0,'refused':0,'wrong':0,'expected_refusals':0}
    for r,a in zip(rows,after):
        if r['expected'] is None:
            outcome='right' if a['status']=='refusal' else 'wrong'
            counts['expected_refusals']+=outcome=='right'
        elif a['status']=='refusal':outcome='refused'
        else:outcome='right' if a['value']==r['expected'] and a.get('unit')==r['unit'] else 'wrong'
        counts[outcome]+=1
    cases=[]
    # Independent construction fixes x and y first, then renders known relations.
    # All six constraint orders are checked; no solver output supplies expectations.
    for x,y in [(F(7,3),F(11,2)),(F(-4),F(9)),(F(17),F(3)),(F(0),F(8))]:
        clauses=[f'The sum of two numbers is {x+y}',f'the first number is {x-y} more than the second number',f'the second number is {y}']
        for order in permutations(clauses):
            q='. '.join(order)+'. What is twice the first number plus 3 times the second number?'
            cases.append((q,str(2*x+3*y)))
        cases.append(('. '.join(clauses[:2])+f'. The first number is {x+1}. Find the second number.',None))
        cases.append((clauses[0]+'. Find the first number.',None))
    for x in [F(5),F(17,3),F(-7)]:
        for a,b in [(F(3),F(2)),(F(7,3),F(1,2)),(F(1,5),F(4,5))]:
            q=f'{a} times a number is {((a-b)*x)} more than {b} times the number. What is 25 percent of the number?'
            cases.append((q,str(x/4)))
            cases.append((q.replace('. What',f'. The number is {x+1}. What'),None))
    case_stream=''.join(json.dumps({'question':q,'expected':e})+'\n' for q,e in cases)
    (TMP/'novel-cases.jsonl').write_text(case_stream)
    predictions=run(BINARY,[q for q,_ in cases])
    failures=[]
    for (q,expected),a in zip(cases,predictions):
        ok=a['status']=='refusal' if expected is None else a['status']=='answer' and F(a['value'])==F(expected) and a.get('unit') is None
        if not ok:failures.append({'question':q,'expected':expected,'actual':a})
    def digest(answers):
        return hashlib.sha256((''.join(json.dumps(canonical(a))+'\n' for a in answers)).encode()).hexdigest()
    report={'synthetic':{'total':len(rows),**counts,'changes':changes,'baseline_canonical_sha256':digest(before),'candidate_canonical_sha256':digest(after)},'novel':{'source':'linearwords-local-development-v1','license':'MIT OR Apache-2.0','total':len(cases),'bytes':len(case_stream.encode()),'sha256':hashlib.sha256(case_stream.encode()).hexdigest(),'failures':failures}}
    (TMP/'checks.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    assert not changes and not failures
if __name__=='__main__':main()
