"""Audit complete frozen reports and compare all changed response/state events."""
import json
from pathlib import Path
root=Path(__file__).with_name('reports');failures=[];summary={}
def read(p):return json.loads((root/p).read_text())
for mode in ['', '-model']:
 b=read(f'baseline-sample{mode}/report.json');c=read(f'candidate-full{mode}/report.json')
 summary['social'+mode]=dict(baseline=[b['right'],b['total']],candidate=[c['right'],c['total']])
 failures += [r for r in c['checks'] if not r['right']]
 before=read(f'baseline-sample{mode}/events.json');after=read(f'candidate-full{mode}/events.json');changes=[]
 assert len(before)==len(after)
 for x,y in zip(before,after):
  assert x['session']==y['session'] and x['inputs']==y['inputs']
  if x!=y:changes.append(dict(inputs=x['inputs'],session=x['session'],baseline=x,candidate=y))
 (root/f'changed-social{mode}.json').write_text(json.dumps(changes,indent=2)+'\n');summary['social'+mode]['changed_events']=len(changes)
for mode in ['', '-model']:
 r=read(f'candidate-fixed-sample{mode}/report.json');assert r['right']==r['total']==126
for version in ['baseline','candidate']:
 assert read(f'{version}-facts/report.json')['checks']==36
 assert read(f'{version}-facts-session/report.json')['checks']==16
assert all(read('candidate-appended-question/report.json')['checks'])
for mode in ['plain','model']:
 c=read(f'candidate-memory-{mode}/report.json');assert c['contracts']==dict(right=93,total=93) and c['original']==dict(right=41,total=41)
 c=read(f'candidate-supplement-{mode}/report.json');assert c['right']==c['total']
 summary['memory-'+mode]='93+41 pass';summary['supplement-'+mode]=[c['right'],c['total']]
for suite in ['calc27','story47']:
 b=read(f'baseline-{suite}.json');c=read(f'candidate-{suite}.json');b.pop('binary_sha256',None);c.pop('binary_sha256',None);assert b==c;summary[suite]='identical'
for suite in ['story-sample','story-full']:
 r=read(suite+'/report.json');assert (root/suite/'baseline.jsonl').read_bytes()==(root/suite/'candidate.jsonl').read_bytes();assert not r['changed_turns'];assert r['baseline']['totals']==r['candidate']['totals'];summary[suite]={k:r[k] for k in ['questions','turns','candidate']}
summary['failures']=failures;(root/'audit.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if not k.startswith('story-')}));assert not failures
