import json,subprocess
from pathlib import Path
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');R=W/'timedmatch-run';O=R/'evidence/cli';O.mkdir(exist_ok=True)
baseline=R/'baseline/glassmind-facts';current=R/'target/release/glassmind-facts'
def call(binary,cmd):return subprocess.run([str(binary),*map(str,cmd)],capture_output=True,check=True).stdout
# Old temporal CLI replay includes supported full witnesses and every old policy case.
cs=json.loads(Path('tools/facts/temporal/contracts.json').read_text());ib=Path('tools/facts/temporal/index.json').read_bytes()
import hashlib
index=O/'index.json';index.write_bytes(ib);asset=O/'temporal.json';results=[]
for c in cs:
    records=[dict(subject='Q1',property=s['mainsnak']['property'],object=s['mainsnak']['datavalue']['value']['id'],source_url='https://www.wikidata.org/wiki/Q1#'+s['mainsnak']['property'],raw_file='0000000000000000.json',raw_sha256='0'*64,statement=s) for s in c['statements']]
    asset.write_text(json.dumps(dict(version=1,index_sha256=hashlib.sha256(ib).hexdigest(),records=records)))
    if '\0' in c['question']:continue # exec cannot represent NUL; library test covers it.
    cmd=['verify-at','--index',index,'--temporal',asset,'--date',c['date'],'--',c['question']]
    a=call(baseline,cmd);b=call(current,cmd);assert a==b,c['name'];results.append(dict(name=c['name'],reply=json.loads(b)))
(O/'old-temporal.json').write_text(json.dumps(results))
cs=json.loads(Path('tools/facts/timed-matching/contracts.json').read_text());results=[]
for c in cs:
    if '\0' in c['question']:continue
    index.write_text(c['index']);asset.write_text(json.dumps(c['asset']))
    reply=json.loads(call(current,['match-at','--index',index,'--temporal',asset,'--date',c['date'],'--',c['question']]))
    assert reply['status']==c['status'],c['name']
    for k,v in c['expected'].items():assert reply[k]==v,(c['name'],k)
    results.append(dict(name=c['name'],reply=reply))
(O/'new-matching.json').write_text(json.dumps(results))
# Full standalone ask command bytes, same fixed first200 then remaining full dev.
rows=(W/'factmatch-source/valid.tsv').read_text().splitlines();before=[];after=[]
for i,row in enumerate(rows):
    q=row.split('\t',3)[3];cmd=['ask','--model',W/'factmatch-source/relation.json','--index',W/'factmatch-source/index.json','--',q]
    a=call(baseline,cmd);b=call(current,cmd);assert a==b,i;before.append(a);after.append(b)
    if i==199:(O/'ask-first200.json').write_text(json.dumps(dict(exact=True,questions=200)))
(O/'baseline-ask.jsonl').write_bytes(b''.join(before));(O/'current-ask.jsonl').write_bytes(b''.join(after))
(O/'report.json').write_text(json.dumps(dict(old_temporal=len(json.loads((O/'old-temporal.json').read_text())),new_matching=len(results),ask_exact=len(rows))))
print((O/'report.json').read_text())
