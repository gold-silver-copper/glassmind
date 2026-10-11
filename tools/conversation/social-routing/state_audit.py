"""Replay each changed social-state snapshot through a fresh talk process.
Expected referents follow the original frozen contracts, with no new fixtures.
"""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary');a=p.parse_args();root=Path(__file__).with_name('reports');contracts=json.loads(Path(__file__).with_name('contracts.json').read_text());checks=[]
for mode in ['', '-model']:
 for event in json.loads((root/f'candidate-full{mode}/events.json').read_text()):
  row=contracts[event['session']]
  if row['type']!='social' or event['inputs']!=[row['text']]:continue
  path=root/'state-audit-session.json';path.write_text(json.dumps(event['state']))
  r=subprocess.run([a.binary,'talk','--json','--session',str(path)],input='Where is Elin?\n',capture_output=True,text=True,check=True,timeout=60)
  reply=json.loads(r.stdout);expected='kitchen' if row['context']=='active' else None
  checks.append(dict(session=event['session'],mode=mode,expected=expected,reply=reply,right=reply.get('answer')==expected))
report=dict(right=sum(c['right'] for c in checks),total=len(checks),checks=checks);(root/'state-audit.json').write_text(json.dumps(report,indent=2)+'\n');print(report['right'],report['total']);assert report['right']==report['total']
