#!/usr/bin/env python3
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parent
binary=pathlib.Path(sys.argv[1]).resolve(); label=sys.argv[2]
assert '/' not in label
out=root/label; out.mkdir(exist_ok=False)
records=[]; identities={}
for suite in ('','supplemental'):
 d=root/suite
 for filename in ('cases.json','store.json'):
  p=d/filename; identities[str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
 for case in json.loads((d/'cases.json').read_text()):
  result=subprocess.run([str(binary),'verify','--index',str(d/'store.json'),case['text']],capture_output=True,text=True,timeout=10)
  try: actual=json.loads(result.stdout)
  except json.JSONDecodeError: actual={}
  status=actual.get('status'); expected=case['expected']
  passed=status in ('unknown','refused') if expected=='not_supported' else status==expected
  if expected=='supported':
   witness=actual.get('witness',{})
   passed &= [witness.get(k) for k in ('subject','property','object')]==case['edge'] and actual.get('scope')=='local_snapshot'
  passed &= result.returncode==0
  records.append({'suite':suite or 'primary','case':case,'actual':actual,'returncode':result.returncode,'stdout':result.stdout,'stderr':result.stderr,'pass':passed})
for p,h in identities.items(): assert hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()==h
summary={'binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'fixtures':identities,'total':len(records),'passed':sum(x['pass'] for x in records),'failures':[x for x in records if not x['pass']]}
(out/'results.json').write_text(json.dumps(records,indent=2,ensure_ascii=False)+'\n')
(out/'summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps(summary,indent=2,ensure_ascii=False))
