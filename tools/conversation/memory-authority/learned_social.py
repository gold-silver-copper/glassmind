"""Post-implementation review check, not part of the preregistered 93 checks.

Frozen before this probe was executed; no runtime edits follow its results.
Specifically exercises learned (rather than literal 'Hello') social fallback.
"""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--model',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
session=a.output/'session.json';session.unlink(missing_ok=True)
lines=['My name is Hal','goodbye',"Actually, it's Nova",'What is my name?']
cmd=[str(a.binary.resolve()),'talk','--json','--session',str(session)]
if a.model:cmd+=['--model',str(a.model)]
r=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
(a.output/'replies.jsonl').write_text(r.stdout);replies=[json.loads(l) for l in r.stdout.splitlines()];assert len(replies)==4
expected=['remembered','intent' if a.model else 'refusal','missing','recalled'];checks=[r['kind']==e for r,e in zip(replies,expected,strict=True)]
state=json.loads(session.read_text());checks += [state['name']=={'value':'Hal','stated_on_turn':1},replies[3].get('source_turn')==1]
if a.model:checks.append(replies[1].get('proposal',{}).get('label')=='goodbye')
report=dict(right=sum(checks),total=len(checks),checks=checks,post_implementation_review=True)
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
