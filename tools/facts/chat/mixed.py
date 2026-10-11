#!/usr/bin/env python3
"""Independent product fixtures; no new factual coverage claim."""
import argparse, hashlib, json, subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--facts-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--model',type=Path)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
checks=[]; session=a.output/'session.json';session.unlink(missing_ok=True)
index_before=hashlib.sha256((a.facts_dir/'index.json').read_bytes()).hexdigest()
def run(lines, persist=True, json_mode=True, facts=True):
    cmd=[str(a.binary),'talk']
    if json_mode:cmd+=['--json']
    if facts:cmd+=['--facts-dir',str(a.facts_dir)]
    if a.model:cmd+=['--model',str(a.model)]
    if persist:cmd+=['--session',str(session)]
    proc=subprocess.run(cmd,input='\n'.join(lines)+'\n',capture_output=True,text=True,check=True,timeout=60)
    return [json.loads(x) for x in proc.stdout.splitlines()] if json_mode else proc.stdout

def check(name, condition, actual=None):
    checks.append(dict(name=name,passed=bool(condition),actual=actual))
    if not condition:
        (a.output/'mixed.json').write_text(json.dumps(checks,indent=2)+'\n')
        raise AssertionError((name,actual))
# Frozen accepted dev question reused only as a routing fixture; no coverage claim.
question='what country is ismo kallio from'
r=run([question],False)[0]
check('known accepted question reaches frozen facts',r['kind']=='fact_answer',r)
expected=r['answers'];check('real singleton QID',len(expected)==1 and expected[0].startswith('Q'))
check('entity and learned relation provenance',r['entity']=='Q11865151' and r['relation']=='P27' and r['provenance']['index_sha256']==index_before,r)
run(['My name is Mira',question])
state=json.loads(session.read_text());check('fact clears name focus',state['last_slot'] is None)
r=run(["Actually, it's Alice",'What is my name?'])
check('bare correction after fact survives restart without editing name',r[0]['kind']=='missing' and 'Mira' in r[1]['text'],r)
run(['My favorite color is blue',question])
r=run(["No, it's green",'What is my favorite color?'])
check('bare correction after fact preserves preference',r[0]['kind']=='missing' and 'blue' in r[1]['text'],r)
r=run(['Actually, my name is Alice','My favorite color is green','What is my name?','What is my favorite color?'])
check('explicit memory override works',r[0]['kind']=='remembered' and 'Alice' in r[2]['text'] and 'green' in r[3]['text'],r)
r=run(['Who invented the imaginary blorptastic engine?',"Actually, it's Violet",'What is my name?'])
check('unsupported fact refuses and clears focus',r[0]['kind']=='refusal' and not r[0].get('answers') and r[1]['kind']=='missing' and 'Alice' in r[2]['text'],r)
r=run(['Calculate 2 + 3 * 4','Mary went to the kitchen.','Where is Mary?'])
check('calculation and established story priority',r[0].get('value')=='14' and r[1]['kind']=='story_fact' and r[2].get('answer')=='kitchen',r)
r=run([question,'Where is Mary?'])
check('facts coexist with established story',r[0].get('answers')==expected and r[1].get('answer')=='kitchen',r)
r=run(['/reset',question,'What is my name?'])
check('reset clears memory but retains frozen facts',r[0]['kind']=='reset' and r[1].get('answers')==expected and r[2]['kind']=='missing',r)
r=run(['My name is Ismo Kallio', 'My favorite country is Finland', 'What is my name?', 'What is my favorite country?'])
check('explicit memory wins for snapshot entities',r[0]['kind']=='remembered' and r[1]['kind']=='remembered' and r[2]['kind']=='recalled' and 'Ismo Kallio' in r[2]['text'] and 'Finland' in r[3]['text'],r)
r=run(['Ismo Kallio is in the kitchen.', 'Where is Ismo Kallio?'])
check('established story overrides indexed entity',r[0]['kind']=='story_fact' and r[1].get('answer')=='kitchen',r)
r=run([question, "Actually, it's Alice", 'What is my name?'])
check('facts with established story cannot correct old personal name',r[0].get('answers')==expected and r[1]['kind']!='remembered' and 'Ismo Kallio' in r[2]['text'],r)
legacy=dict(turn=1,name=dict(value='Ada',stated_on_turn=1),preferences={},last_slot='Name',last_user_utterance='My name is Ada',last_correction=None)
session.write_text(json.dumps(legacy))
r=run(['What is my name?',question])
r2=run(["Actually, it's Bob",'What is my name?'])
check('legacy session and persisted fact focus',r[0]['source_turn']==1 and r2[0]['kind']=='missing' and 'Ada' in r2[1]['text'],r2)
text=run([question],False,False)
check('human attribution and links',all(x in text for x in ['local Wikidata snapshot suggests','https://www.wikidata.org/wiki/']),text)
check('index immutable',index_before==hashlib.sha256((a.facts_dir/'index.json').read_bytes()).hexdigest())
# Each invalid directory is independent; the loader must fail before emitting replies.
for name,files in [('empty',{}),('index-only',{'index.json':'index.json'}),('model-only',{'relation.json':'relation.json'}),('index-mismatch',{'relation.json':'relation.json'}),('model-mismatch',{'index.json':'index.json'}),('labels-mismatch',{'index.json':'index.json','relation.json':'relation.json'})]:
    d=a.output/name;d.mkdir(exist_ok=True)
    (d/'manifest.json').write_bytes((a.facts_dir/'manifest.json').read_bytes())
    for dest,src in files.items():
        target=d/dest;target.unlink(missing_ok=True);target.symlink_to((a.facts_dir/src).resolve())
    if name=='index-mismatch':(d/'index.json').write_text('{"entities":{}}')
    if name=='model-mismatch':(d/'relation.json').write_text('{}')
    if name=='labels-mismatch':(d/'labels.json').write_text(json.dumps(dict(index_sha256='wrong',source='Wikidata',license='CC0-1.0',labels={})))
    r=subprocess.run([str(a.binary),'talk','--json','--facts-dir',str(d)],input=question+'\n',text=True,capture_output=True,timeout=60)
    check(name+' fails cleanly',r.returncode!=0 and not r.stdout and bool(r.stderr),r.stderr)
# Default mode must work even if its working directory has broken factual assets.
r=subprocess.run([str(a.binary),'talk','--json'],input='My name is Mira\n',cwd=a.output/'model-mismatch',capture_output=True,text=True,check=True,timeout=60)
check('default ignores facts assets',json.loads(r.stdout)['kind']=='remembered')
# No display labels must remain a supported, explicit QID-only presentation.
d=a.output/'no-labels';d.mkdir(exist_ok=True)
for name in ['manifest.json','index.json','relation.json']:
    target=d/name;target.unlink(missing_ok=True);target.symlink_to((a.facts_dir/name).resolve())
r=subprocess.run([str(a.binary),'talk','--facts-dir',str(d)],input=question+'\n',capture_output=True,text=True,check=True,timeout=60)
check('missing labels explicitly disclose QIDs',all(x in r.stdout for x in ['English label unavailable',expected[0],'https://www.wikidata.org/wiki/']),r.stdout)
# A display-only label cannot become a linker alias or alter answer IDs.
(d/'labels.json').write_text(json.dumps(dict(index_sha256=index_before,source='fixture display labels',license='CC0-1.0',labels={'Q11865151':'Zzzdisplayonly'})))
r=subprocess.run([str(a.binary),'talk','--json','--facts-dir',str(d)],input=question+'\n/reset\nwhat country is Zzzdisplayonly from\n',capture_output=True,text=True,check=True,timeout=60)
v=[json.loads(x) for x in r.stdout.splitlines()]
check('display labels do not link or change IDs',v[0].get('answers')==expected and v[2]['kind']=='refusal' and not v[2].get('answers'),v)
(d/'labels.json').write_text(json.dumps(dict(index_sha256=index_before,source='fixture',license='CC0-1.0',labels={'Q999999999999999':'outside snapshot'})))
r=subprocess.run([str(a.binary),'talk','--facts-dir',str(d)],input=question+'\n',capture_output=True,text=True,timeout=60)
check('labels outside snapshot rejected',r.returncode!=0 and not r.stdout,r.stderr)
# Same-sized but changed bytes exercise the compatibility fingerprint, too.
d=a.output/'same-size-mismatch';d.mkdir(exist_ok=True)
(d/'manifest.json').write_bytes((a.facts_dir/'manifest.json').read_bytes())
original=(a.facts_dir/'index.json').read_bytes()
(d/'index.json').write_bytes(original.replace(b'Boston', b'Bostom', 1))
assert (d/'index.json').stat().st_size==len(original)
target=d/'relation.json';target.unlink(missing_ok=True);target.symlink_to((a.facts_dir/'relation.json').resolve())
r=subprocess.run([str(a.binary),'talk','--facts-dir',str(d)],input=question+'\n',capture_output=True,text=True,timeout=60)
check('same-size changed index rejected',r.returncode!=0 and not r.stdout and 'does not match' in r.stderr,r.stderr)
(a.output/'mixed.json').write_text(json.dumps(dict(total=len(checks),right=sum(x['passed'] for x in checks),checks=checks),indent=2)+'\n')
print(json.dumps(dict(total=len(checks),right=sum(x['passed'] for x in checks))))
