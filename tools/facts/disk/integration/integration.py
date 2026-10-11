"""Central-build native CLI parity plus unchanged legacy talk routing checks."""
from pathlib import Path
import json,hashlib,subprocess
ROOT=Path('/home/gsc/code/glassmind');WORK=Path('/home/gsc/.cache/glassmind/continuous-20261010');HERE=Path(__file__).resolve().parent
BIN=ROOT/'target/release';BUNDLE=WORK/'trainindex-run/tmp/trainindex/bundle1000';ART=WORK/'linkquality-run/artifacts'
checks=[]
for split,n in [('train',19481),('valid',2821)]:
 expected=[json.loads(s) for s in (HERE/f'{split}-parity.out').read_text().splitlines()]
 for mode in ['answer','set']:
  args=[str(BIN/'glassmind-facts'),'predict-disk','--model',str(BUNDLE/'relation.json'),'--database',str(HERE/'native999.sqlite')]
  if mode=='set':args.append('--set')
  p=subprocess.run(args,input=(ART/f'{split}.questions').read_bytes(),capture_output=True,check=True,timeout=120)
  output=[json.loads(s) for s in p.stdout.splitlines()];assert len(output)==n
  assert output==[e[mode] for e in expected]
  (HERE/f'integrated-{split}-{mode}.jsonl').write_bytes(p.stdout)
  checks.append({'split':split,'mode':mode,'rows':n,'matched':n})
for mode in ['plain','intent']:
 out=HERE/f'main-{mode}';args=['python3',str(ROOT/'tools/facts/chat/mixed.py'),'--binary',str(BIN/'glassmind'),'--facts-dir',str(HERE/'legacy-main-bundle'),'--output',str(out)]
 if mode=='intent':args+=['--model','/home/gsc/.cache/glassmind/models/everyday/intent.json']
 p=subprocess.run(args,cwd=ROOT,capture_output=True,check=True,timeout=300)
 checks.append({'main_mode':mode,'result':json.loads(p.stdout)})
 print(p.stdout.decode().strip(),flush=True)
report={'source_commit':'3de724e','checks':checks,'binaries':[]}
for name in ['glassmind','glassmind-facts']:
 p=BIN/name
 with p.open('rb') as f:h=hashlib.file_digest(f,'sha256').hexdigest()
 report['binaries'].append({'path':str(p),'bytes':p.stat().st_size,'sha256':h})
(HERE/'integration-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
