"""Compare retained native verification, never benchmark correctness."""
import argparse, collections, hashlib, json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('evidence',type=Path);a=p.parse_args();out={}
for mode in ('baseline','candidate'):
 out[mode]={}
 for name in ('authored','supplement'):
  rows=json.loads((a.evidence/f'{mode}-{name}'/'results.json').read_text())
  out[mode][name]={'total':len(rows),'checks_passed':sum(r['ok'] for r in rows),'positive_exact':sum(r['kind']=='positive' and r['changed'] and r['exact'] and r['verify']==0 for r in rows),'negative_leaks':sum(r['kind']=='negative' and r['changed'] for r in rows)}
 for name in ('sample','full'):
  out[mode][name]=json.loads((a.evidence/f'{mode}-{name}'/'report.json').read_text())['totals']
for name in ('sample','full'):
 b=json.loads((a.evidence/f'baseline-{name}'/'report.json').read_text());c=json.loads((a.evidence/f'candidate-{name}'/'report.json').read_text())
 assert b['inputs_sha256']==c['inputs_sha256']
 assert [r['source'] for r in b['details']]==[r['source'] for r in c['details']]
 out[name+'_transitions']=[{'source':x['source'],'before':x,'after':y} for x,y in zip(b['details'],c['details']) if x!=y]
(a.evidence/'summary.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
