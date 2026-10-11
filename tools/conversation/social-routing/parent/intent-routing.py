"""Audit main-talk routing on the unchanged public CLINC validation split.
Retention of previously correct learned proposals, not a new classifier score.
"""
from pathlib import Path
import argparse,collections,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('out',type=Path);p.add_argument('--baseline',type=Path);a=p.parse_args();a.out.mkdir(exist_ok=False)
data=Path('/home/gsc/.cache/glassmind/data/clinc150/val.jsonl');model=Path('/home/gsc/.cache/glassmind/models/everyday/intent.json');rows=[json.loads(l) for l in data.read_text().splitlines()];assert len(rows)==3100
assert all('\n' not in r['text'] for r in rows)
inputs=''.join('/reset\n'+r['text']+'\n' for r in rows)
raw=subprocess.run([str(a.binary.resolve()),'talk','--json','--model',str(model)],input=inputs.encode(),capture_output=True,check=True,timeout=550).stdout
(a.out/'replies.jsonl').write_bytes(raw);replies=[json.loads(x) for x in raw.splitlines()];assert len(replies)==6200
replies=replies[1::2]
def correct(row,reply):return reply.get('kind')=='intent' and reply.get('proposal',{}).get('accepted') is True and reply['proposal']['label']==row['label']
records=[dict(index=i,text=row['text'],gold=row['label'],reply=reply,correct_learned_proposal=correct(row,reply)) for i,(row,reply) in enumerate(zip(rows,replies,strict=True))]
report=dict(scope='Main talk on CLINC150 validation with per-row resets; only learned-proposal correctness/retention is measured. Typed memory/social replies are not graded as classifier failures.',rows=3100,binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),data_sha256=hashlib.sha256(data.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),reply_kinds=dict(collections.Counter(x['reply']['kind'] for x in records)),correct_learned_proposals=sum(x['correct_learned_proposal'] for x in records))
if a.baseline:
 old=json.loads((a.baseline/'records.json').read_text());assert len(old)==len(records)
 losses=[];gains=[];changes=[]
 for x,y in zip(old,records,strict=True):
  assert x['text']==y['text'] and x['gold']==y['gold']
  if x['correct_learned_proposal'] and not y['correct_learned_proposal']:losses.append(dict(index=x['index'],text=x['text'],gold=x['gold'],before=x['reply'],after=y['reply']))
  if not x['correct_learned_proposal'] and y['correct_learned_proposal']:gains.append(dict(index=x['index'],text=x['text'],gold=x['gold'],before=x['reply'],after=y['reply']))
  if x['reply']!=y['reply']:changes.append(x['index'])
 report.update(previous_correct=sum(x['correct_learned_proposal'] for x in old),lost_correct=len(losses),new_correct=len(gains),changed_rows=len(changes),losses_by_label=dict(collections.Counter(x['gold'] for x in losses)),gains_by_label=dict(collections.Counter(x['gold'] for x in gains)))
 (a.out/'transitions.json').write_text(json.dumps(dict(losses=losses,gains=gains,changed_indices=changes),indent=2)+'\n')
(a.out/'records.json').write_text(json.dumps(records,indent=2)+'\n');(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
