#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
import run as r,json,subprocess
from pathlib import Path
b=r.BIN/'reading-qa-syntax';base=[str(b),'--syntax',str(r.BASE/'syntax.bin'),'predict','--model',str(r.RUN/'model.json')]
checks={}
def invoke(name,rows,cmd=base):
    p=r.RUN/('contract.'+name+'.jsonl');r.q.lines(p,rows);return subprocess.run(cmd+['--input',str(p)],capture_output=True,text=True,timeout=30)
for field,value in [('id','x'),('title','x'),('answers',[]),('heads',[0]),('forms',['x'])]:
    p=invoke(field,[dict(context='Élodie thanked Björn.',question='Who thanked Björn?',**{field:value})]);assert p.returncode!=0 and not p.stdout;checks['reject_'+field]=True
p=invoke('missing',[dict(context='x',question='y')],[str(b),'predict','--model',str(r.RUN/'model.json')]);assert p.returncode!=0 and not p.stdout;checks['missing_companion']=True
p=invoke('wrong',[dict(context='x',question='y')],[str(b),'--syntax',str(r.BASE/'model.json'),'predict','--model',str(r.RUN/'model.json')]);assert p.returncode!=0 and not p.stdout;checks['wrong_companion']=True
m=json.loads((r.RUN/'model.json').read_text());m['syntax_sha256']='0'*64;r.write(r.RUN/'incompatible.json',m)
p=invoke('pair',[dict(context='x',question='y')],[str(b),'--syntax',str(r.BASE/'syntax.bin'),'predict','--model',str(r.RUN/'incompatible.json')]);assert p.returncode!=0 and not p.stdout;checks['wrong_pair']=True
rows=[dict(context='Élodie thanked Björn. Élodie thanked Björn.',question='Who thanked Björn?'),dict(context='x'*65537,question='Which x?'),dict(context=' '.join(['word']*129),question='Which word?'),dict(context='Mira did not thank Nora.',question='Who did Mira not thank?')]
p=invoke('valid',rows);assert p.returncode==0,p.stderr
ps=[json.loads(l) for l in p.stdout.splitlines()];assert len(ps)==4
for row,out in zip(rows,ps):
    assert out['semantic_status']=='unverified';r.q.proposal(row,out)
    if out['proposal']:assert row['context'].encode()[out['proposal']['start']:out['proposal']['end']].decode()
assert ps[1]['status']=='refused' and ps[1]['proposal'] is None
checks['utf8_bounds_missing_features_unverified']=True
p2=invoke('deterministic',rows);assert p.stdout==p2.stdout;checks['deterministic']=True
r.write(r.REPORT/'contracts.json',checks);print(json.dumps(checks))
