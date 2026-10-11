# Offline report completion after a baseline-directory typo; no inference repeated.
# Original scoring logic and data unchanged. Original adapter SHA256 8f8c38c7fea00195956270807e191963dff7147ad9ce169dc576eff028f92700
"""All training-partition bAbI dev turns through talk, with explicit model mode."""
import argparse,collections,hashlib,importlib.util,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('out',type=Path);p.add_argument('--model',type=Path);p.add_argument('--baseline',type=Path);a=p.parse_args();assert a.out.is_dir()
spec=importlib.util.spec_from_file_location('babi',Path('/home/gsc/code/glassmind/tools/stories/babi.py'));b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
rows=list(b.items(Path('/home/gsc/.cache/glassmind/data/babi/en'),'dev',0));assert len(rows)==3987
lines=[];indices=[]
for row in rows:
 lines+=['/reset',*row['story'],row['question']];indices.append(len(lines)-1)
args=[str(a.binary.resolve()),'talk','--json']
if a.model:args+=['--model',str(a.model)]
raw=(a.out/'replies.jsonl').read_bytes()
(a.out/'replies.jsonl').write_bytes(raw);replies=[json.loads(x) for x in raw.splitlines()];assert len(replies)==len(lines)
totals=collections.Counter();outcomes=[]
for row,i in zip(rows,indices,strict=True):
 reply=replies[i];ans=reply.get('answer');status='refusal' if ans is None else 'correct' if b.normalize(ans,row['task'])==b.normalize(row['expected'],row['task']) else 'wrong'
 totals[status]+=1;outcomes.append(dict(id=row['id'],outcome=status,reply=reply))
report=dict(dataset='bAbI-v1.2-training-partition-through-talk',split='dev',total=len(rows),utterances=len(lines),totals=dict(totals),binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(a.model.read_bytes()).hexdigest() if a.model else None,input_sha256=hashlib.sha256(('\n'.join(lines)+'\n').encode()).hexdigest(),outputs_sha256=hashlib.sha256(raw).hexdigest(),item_ids_sha256=b.digest('\n'.join(r['id'] for r in rows).encode()))
if a.baseline:
 before=(a.baseline/'replies.jsonl').read_bytes();old=json.loads((a.baseline/'report.json').read_text());assert old['input_sha256']==report['input_sha256'] and old['model_sha256']==report['model_sha256']
 report['byte_identical_baseline']=before==raw
 oldrows=json.loads((a.baseline/'outcomes.json').read_text());report['outcome_changes']=[dict(id=x['id'],before=x['outcome'],after=y['outcome']) for x,y in zip(oldrows,outcomes,strict=True) if x['outcome']!=y['outcome']]
(a.out/'outcomes.json').write_text(json.dumps(outcomes,indent=2)+'\n');(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
