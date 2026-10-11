"""Verify retained reports and identities, without rerunning or changing scorers."""
from pathlib import Path
import hashlib,json,os
root=Path('tools/conversation/route-authority');r=Path(os.environ.get('ROUTE_REPORTS',str(root/'reports')))
def read(name):return json.loads((r/name).read_text())
checks={}
for mode in ['plain','model']:
 for split,n in [('sample',126),('full',490)]:
  x=read(f'candidate-old-{split}-{mode}/report.json');checks[f'old-{split}-{mode}']=x['right']==x['total']==n
 x=read(f'candidate-memory-{mode}/report.json');checks[f'memory-{mode}']=x['contracts']==dict(right=93,total=93) and x['original']==dict(right=41,total=41)
 x=read(f'candidate-goodbye-{mode}/report.json');n=6 if mode=='plain' else 7;checks[f'goodbye-{mode}']=x['right']==x['total']==n
 suffix='' if mode=='plain' else '-model'
 for split,n in [('sample',59),('',122)]:
  name='candidate-boundary'+('-sample' if split else '')+suffix
  x=read(f'{name}/report.json');checks[name]=x['right']==x['total']==n
x=read('candidate-stale/report.json');checks['stale']=x['checks']==[True,True]
x=read('calc27.json');checks['calc27']=x['right']==x['total']==27
x=read('story47.json');checks['story47']=x['total']==x['totals']['right']==47
checks['facts36']=read('facts/report.json')['checks']==36;checks['facts-session16']=read('facts-session/report.json')['checks']==16
x=read('intent-full/report.json');checks['routing-retention']=x['rows']==3100 and x['previous_correct']==1303 and x['lost_correct']==0 and x['new_correct']==2 and x['correct_learned_proposals']==1305
x=read('changed-routing-audit.json');checks['changed-row-audit']=x['changed_rows']==19 and x['lost_correct']==0 and x['new_correct']==2 and x['new_wrong']==0
x=read('story-full/report.json');checks['story-full']=x['questions']==3987 and x['turns']==47625 and x['changed_turns']==[] and (r/'story-full/baseline.jsonl').read_bytes()==(r/'story-full/candidate.jsonl').read_bytes()
for source in [root/'verified-inputs.json',root/'identities.json']:
 checks[source.name]=all(Path(p).stat().st_size==v['bytes'] and hashlib.sha256(Path(p).read_bytes()).hexdigest()==v['sha256'] for p,v in json.loads(source.read_text()).items())
for f in [root/'FROZEN.sha256',r/'source.sha256']:
 checks[f.name]=all(hashlib.sha256(Path(line.split(maxsplit=1)[1]).read_bytes()).hexdigest()==line.split()[0] for line in f.read_text().splitlines())
checks['unit-tests']=all(f'{n} passed; 0 failed' in (r/name).read_text() for name,n in [('test-everyday.log',3),('test-stories.log',4),('test-memory.log',4)])
checks['build']='Finished `release`' in (r/'build.log').read_text();checks['clippy']='Finished `dev`' in (r/'clippy.log').read_text()
report=dict(status='PASS' if all(checks.values()) else 'REJECT',binary_sha256=hashlib.sha256(Path('../routeauthority-run/glassmind-candidate').read_bytes()).hexdigest(),checks=checks)
(r/'acceptance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));assert all(checks.values())
