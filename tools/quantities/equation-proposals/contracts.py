#!/usr/bin/env python3
"""Small public authored black-box boundary/semantic probes, no corpus reads."""
import hashlib,json,os,subprocess
from pathlib import Path
run=Path(os.environ['PROPOSAL_RUN']);binary=run/'glassmind-quantities';model=run/'model.json'
def invoke(data):
    return subprocess.run([str(binary),'--propose-model',str(model),'--jsonl'],input=data,text=True,capture_output=True,timeout=60)
qs=['Éva has 1,200.50 and 3/4.', '-0.5 and +.25', '1e3 plus 2','−2 plus 3','- 2 plus 3','1,23 plus 4','１２ plus 3','1--2','x'*4097,'1 '*17]
p=invoke(''.join(json.dumps(q)+'\n' for q in qs));assert p.returncode==0,p.stderr
outs=[json.loads(x) for x in p.stdout.splitlines()];assert len(outs)==len(qs)
for q,o in zip(qs,outs):
    assert o['status'] in ['proposed','refusal']
    if o['status']=='proposed':
        assert o['semantics']=='unverified'
        for n in o['numbers']:assert q.encode()[n['start']:n['end']].decode()==n['literal']
for o in outs[2:]:assert o['status']=='refusal',o
for invalid in [json.dumps({'Problem':'8 and 2','correct':'a'})+'\n',' '*32769+'\n',json.dumps({'question':'8 and 2','options':[]})+'\n']:
    assert invoke(invalid).returncode!=0
# Contrasts are diagnostic, never used for fitting or threshold changes.
contrasts=[
 ('A crate contains 8 packs of 2 bolts. How many bolts are in the crate?','16'),
 ('A crate contains 8 bolts, of which 2 are defective. How many bolts are usable?','6'),
 ('Mira has 8 coins and gives away 2 coins. How many coins does Mira have now?','6'),
 ('Mira has 8 coins and receives 2 coins. How many coins does Mira have now?','10'),
 ('A machine makes 8 parts in 2 hours. How many parts does it make per hour?','4'),
 ('A machine makes 8 parts per hour for 2 hours. How many parts does it make?','16')]
p=invoke(''.join(json.dumps(q)+'\n' for q,_ in contrasts));assert p.returncode==0,p.stderr
cs=[]
for (q,expected),line in zip(contrasts,p.stdout.splitlines()):
    o=json.loads(line);cs.append({'question':q,'expected':expected,'actual':o,'numeric_match':o.get('value')==expected})
result={'boundary_checks_pass':True,'boundaries':list(zip(qs,outs)),'contrasts':cs,'contrast_numeric_matches':sum(c['numeric_match'] for c in cs),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'model_sha256':hashlib.sha256(model.read_bytes()).hexdigest()}
(run/'authored-contracts.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['boundaries','contrasts']}))
