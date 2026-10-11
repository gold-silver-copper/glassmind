"""Offline audit of every changed CLINC dev route; no runtime data injection."""
import hashlib,json,os
from pathlib import Path
root=Path(os.environ.get('ROUTE_REPORTS','tools/conversation/route-authority/reports'))
old=json.loads(Path('../coordinator/socialrouting-review/intent-baseline/records.json').read_text())
new=json.loads((root/'intent-full/records.json').read_text())
changed=[]
for b,c in zip(old,new,strict=True):
 assert (b['text'],b['gold'])==(c['text'],c['gold'])
 if b['reply']==c['reply']:continue
 p=c['reply'].get('proposal',{})
 wrong=c['reply']['kind']=='intent' and p.get('accepted') is True and p['label']!=c['gold']
 before=b['reply'].get('proposal',{})
 inherited_wrong=b['reply']['kind']=='intent' and before.get('accepted') is True and before.get('label')==p.get('label')
 changed.append(dict(index=c['index'],text=c['text'],gold=c['gold'],before=b['reply'],after=c['reply'],lost_correct=b['correct_learned_proposal'] and not c['correct_learned_proposal'],new_correct=c['correct_learned_proposal'] and not b['correct_learned_proposal'],new_wrong=wrong and not inherited_wrong,category='gain' if c['correct_learned_proposal'] and not b['correct_learned_proposal'] else 'lost_correct' if b['correct_learned_proposal'] and not c['correct_learned_proposal'] else 'inherited_wrong' if wrong else 'typed_or_refused'))
report=dict(changed_rows=len(changed),lost_correct=sum(c['lost_correct'] for c in changed),new_correct=sum(c['new_correct'] for c in changed),new_wrong=sum(c['new_wrong'] for c in changed),rows=changed)
(root/'changed-routing-audit.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='rows'}))
