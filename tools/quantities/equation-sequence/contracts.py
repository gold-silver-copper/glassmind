#!/usr/bin/env python3
"""Reuse the previous six public authored contrasts without changing expectations."""
import hashlib,json,os,subprocess
from pathlib import Path
RUN=Path(os.environ['SEQUENCE_RUN']);OLD=Path('/home/gsc/.cache/glassmind/continuous-20261010/equationproposals-run/frozen-candidate')
SCOPE=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
def invoke(data,model=None):
    return subprocess.run(SCOPE+[str(RUN/'equation_sequence'),str(model or RUN/'model.json')],input=data,text=True,capture_output=True,timeout=610)
def predict(qs):
    p=invoke(''.join(json.dumps(q)+'\n' for q in qs));assert p.returncode==0,p.stderr;out=[json.loads(l) for l in p.stdout.splitlines()];assert len(out)==len(qs);return out
old=json.loads((OLD/'authored-contracts.json').read_text());qs=[q for q,o in old['boundaries']];out=predict(qs)
for q,o in zip(qs,out):
    if o['status']=='proposed':
        assert o['semantics']=='unverified';assert o['model_sha256']==hashlib.sha256((RUN/'model.json').read_bytes()).hexdigest()
        for n in o['numbers']:assert q.encode()[n['start']:n['end']].decode()==n['literal']
for o in out[2:]:assert o['status']=='refusal'
for bad in [json.dumps({'Problem':'8 and 2','correct':'a'})+'\n',' '*32769+'\n',json.dumps({'question':'8 and 2','options':[]})+'\n']:assert invoke(bad).returncode!=0
bad=RUN/'bad-model.json';bad.write_text('{"schema":"unknown","provenance":"test","weights":[]}')
assert invoke('"8 and 2"\n',bad).returncode!=0
bad.write_bytes(b' '*32_000_001);assert invoke('"8 and 2"\n',bad).returncode!=0
bad.unlink()
contrasts=[]
for original,o in zip(old['contrasts'],predict([c['question'] for c in old['contrasts']])):
    contrasts.append({'question':original['question'],'expected':original['expected'],'actual':o,'numeric_match':o.get('value')==original['expected']})
report={'boundaries_pass':True,'malformed_model_and_oversized_row_pass':True,'contrasts':contrasts,'contrast_numeric_matches':sum(c['numeric_match'] for c in contrasts)}
(RUN/'authored-contracts.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='contrasts'}))
