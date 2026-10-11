#!/usr/bin/env python3
import json,pathlib,subprocess,sys
base=pathlib.Path(__file__).resolve().parent
rows=json.loads((base/'reports/train-input-provenance.json').read_text())
# Request-only parser probe; no workspace execution or semantic correctness score.
r=subprocess.run([sys.argv[1]],input='\n'.join(x['text'] for x in rows)+'\n',text=True,capture_output=True,check=True)
result=[dict(id=x['id'],parsed=y not in ['REFUSED','UNRECOGNIZED'],target=y) for x,y in zip(rows,r.stdout.splitlines(),strict=True)]
contracts=json.loads((base/'contracts.json').read_text())
inputs=[x[0] for x in contracts['accept']]+contracts['refuse']
r=subprocess.run([sys.argv[1]],input='\n'.join(x.replace('\n',' ') for x in inputs)+'\n',text=True,capture_output=True,check=True)
expected=[x[1] for x in contracts['accept']]+[None]*len(contracts['refuse'])
checks=[dict(text=x,result=y,ok=y==e if e else y in ['REFUSED','UNRECOGNIZED']) for x,y,e in zip(inputs,r.stdout.splitlines(),expected,strict=True)]
print(json.dumps(dict(note='Parser coverage only; NOT test-verified implementation correctness.',total=len(result),parsed=sum(x['parsed'] for x in result),rows=result,contracts=checks),indent=2,ensure_ascii=False))
raise SystemExit(0 if all(x['ok'] for x in checks) else 1)
