#!/usr/bin/env python3
import collections,json,pathlib,sys
base=json.loads(pathlib.Path(sys.argv[1]).read_text());candidate=json.loads(pathlib.Path(sys.argv[2]).read_text())
def repair(r):return any(x.startswith('repair_') for x in r['understood']+r['guesses'])
def correct(r):
 e=r['expected'];u=r['understood']
 if e=='refuse':return not repair(r)
 if e in ('build','test'):e='repair_build' if e=='build' else 'repair_tests'
 if e.startswith('status'):return any(x in (e,e.replace('status','problems')) for x in u) and not repair(r)
 return e in u
summary={}
for group in sorted({r['group'] for r in base}):
 b=[r for r in base if r['group']==group];c=[r for r in candidate if r['group']==group]
 transitions=[dict(index=x['index'],expected=x['expected'],text=x['text'],baseline=x['understood'],candidate=y['understood'],baseline_guesses=x['guesses'],candidate_guesses=y['guesses']) for x,y in zip(b,c) if x['understood']!=y['understood'] or x['guesses']!=y['guesses']]
 summary[group]=dict(total=len(b),baseline_expected=sum(map(correct,b)),candidate_expected=sum(map(correct,c)),baseline_repair_readings=sum(map(repair,b)),candidate_repair_readings=sum(map(repair,c)),lost_expected=[x['index'] for x,y in zip(b,c) if correct(x) and not correct(y)],candidate_failures=[dict(index=r['index'],expected=r['expected'],text=r['text'],understood=r['understood'],guesses=r['guesses']) for r in c if not correct(r)],transitions=transitions)
 if group=='train172':
  summary[group]['by_kind']={k:dict(total=sum(r['expected']==k for r in b),baseline=sum(correct(r) for r in b if r['expected']==k),candidate=sum(correct(r) for r in c if r['expected']==k)) for k in ['build','test']}
print(json.dumps(summary,indent=2))
