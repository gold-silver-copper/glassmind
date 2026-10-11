#!/usr/bin/env python3
"""Fresh-context talk vs frozen facts. Only question text is sent to either runtime."""
import argparse, collections, hashlib, json, subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary',type=Path,required=True)
p.add_argument('--facts-binary',type=Path,required=True)
p.add_argument('--predictor',type=Path,required=True)
p.add_argument('--facts-dir',type=Path,required=True)
p.add_argument('--valid',type=Path,required=True)
p.add_argument('--limit',type=int)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args(); a.output.mkdir(parents=True,exist_ok=True)
assert a.valid.name=='valid.tsv', 'development split only'
rows=[x.split('\t',3) for x in a.valid.read_text().splitlines()][:a.limit]
questions=''.join(r[3]+'\n' for r in rows)
(a.output/'questions.txt').write_text(questions)
def run(args, text=None):
    return subprocess.run(list(map(str,args)),input=text,text=True,capture_output=True,timeout=550,check=True).stdout
baseline=run([a.predictor,a.facts_dir/'relation.json',a.facts_dir/'index.json'],questions)
(a.output/'baseline.jsonl').write_text(baseline)
old=[json.loads(x) for x in baseline.splitlines()]
inputs=''.join('/reset\n'+r[3]+'\n' for r in rows)
raw=run([a.binary,'talk','--json','--facts-dir',a.facts_dir],inputs)
(a.output/'talk.jsonl').write_text(raw)
all_replies=[json.loads(x) for x in raw.splitlines()]
assert len(all_replies)==len(rows)*2
assert all(x['kind']=='reset' for x in all_replies[::2])
replies=all_replies[1::2]
new=[dict(entity=x.get('entity'),relation=x.get('relation'),answers=x.get('answers',[]),refusal=x.get('refusal') or (None if x.get('kind')=='fact_answer' else 'other route refused')) for x in replies]
(a.output/'predictions.jsonl').write_text(''.join(json.dumps(x)+'\n' for x in new))
# Same classification as the frozen scorer. Also run that scorer independently.
def score(row, pred):
    if pred['answers']==[row[2]]:return 'right'
    return 'refused' if pred['refusal'] else 'wrong'
counts={name:dict(collections.Counter(score(r,x) for r,x in zip(rows,preds,strict=True))) for name,preds in [('baseline',old),('talk',new)]}
changed=[dict(row=i+1,question=r[3],before=o,after=n,reply=reply) for i,(r,o,n,reply) in enumerate(zip(rows,old,new,replies,strict=True)) if o['answers']!=n['answers']]
accepted=sum(bool(x['answers']) for x in old)
scorer_args=[a.facts_binary,'score','--input',a.valid,'--model',a.facts_dir/'relation.json','--index',a.facts_dir/'index.json']
if a.limit: scorer_args+=['--limit',a.limit]
scorer=json.loads(run(scorer_args))
assert all(scorer[k]==counts['baseline'].get(k,0) for k in ['right','refused','wrong'])
files={str(path):dict(bytes=path.stat().st_size,sha256=hashlib.sha256(path.read_bytes()).hexdigest()) for path in [a.binary,a.predictor,a.facts_dir/'manifest.json',a.facts_dir/'index.json',a.facts_dir/'relation.json',a.valid,a.output/'questions.txt',a.output/'baseline.jsonl',a.output/'predictions.jsonl']}
report=dict(total=len(rows),counts=counts,frozen_scorer=scorer,baseline_accepted=accepted,preserved_accepted=sum(bool(o['answers']) and o['answers']==n['answers'] for o,n in zip(old,new)),answer_id_changes=changed,files=files,scope='500 validation-subject-seeded entities; no new factual coverage; question text only with /reset per question')
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in {'files','answer_id_changes'}}))
if changed: raise SystemExit(f'{len(changed)} answer-ID changes')
