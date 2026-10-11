"""Compare original authored fact-set requests, bytes and final session state."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('baseline',type=Path);p.add_argument('candidate',type=Path);p.add_argument('model',type=Path);p.add_argument('bundle',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
f=json.loads(Path('tools/facts/sets/contracts.json').read_text())
lines=['My name is Nia','I prefer tea']+[r[0] for r in f['positive']]+f['negative'];report=[]
for model in [False,True]:
 for bundle in [False,True]:
  raw=[];states=[]
  for label,binary in [('baseline',a.baseline),('candidate',a.candidate)]:
   tag=f'{label}-model{model}-facts{bundle}';session=a.output/(tag+'-session.json');session.unlink(missing_ok=True)
   command=[str(binary.resolve()),'talk','--json','--session',str(session)]
   if model:command+=['--model',str(a.model)]
   if bundle:command+=['--facts-dir',str(a.bundle)]
   r=subprocess.run(command,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60).stdout
   (a.output/(tag+'.jsonl')).write_text(r);raw.append(r);states.append(session.read_bytes())
  assert raw[0]==raw[1] and states[0]==states[1],(model,bundle)
  report.append(dict(model=model,bundle=bundle,replies=len(lines),reply_bytes_identical=True,session_bytes_identical=True,sha256=hashlib.sha256(raw[0].encode()).hexdigest()))
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
