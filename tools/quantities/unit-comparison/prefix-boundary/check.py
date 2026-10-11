from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--model',type=Path);a=p.parse_args()
a.out.mkdir(exist_ok=False);stem='Is 1 kg equal to 1000 g?';cases=[]
for label,space in [('ascii',' '),('unicode','\u2003')]:
    gap=4096-len(stem.encode());payload=stem+space*(gap//len(space.encode()))+' '*(gap%len(space.encode()))
    assert len(payload.encode())==4096
    for excess in [0,1]:
        text=payload+' '*excess
        for prefix in ['', 'Compare quantities:', 'Compare quantities: ']:
            cases.append(dict(name=f'{label}-{excess}-{len(prefix)}',line=prefix+text,wanted='comparison' if not excess else 'refusal'))
for prefix in ['', 'Compare quantities: ']:cases.append(dict(name=f'nul-{len(prefix)}',line=prefix+stem+'\0',wanted='refusal'))
cmd=[str(a.binary),'talk','--json']
if a.model:cmd+=['--model',str(a.model)]
r=subprocess.run(cmd,input=''.join(c['line']+'\n' for c in cases),text=True,capture_output=True,check=True,timeout=60)
rows=[json.loads(s) for s in r.stdout.splitlines()];assert len(rows)==len(cases)
(a.out/'inputs.json').write_text(json.dumps(cases,ensure_ascii=False)+'\n');(a.out/'outputs.jsonl').write_text(r.stdout)
results=[dict(name=c['name'],wanted=c['wanted'],actual=v.get('status'),ok=v.get('kind')=='quantity_comparison' and v.get('status')==c['wanted']) for c,v in zip(cases,rows)]
report=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),total=len(cases),passed=sum(x['ok'] for x in results),results=results)
(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));raise SystemExit(0 if all(x['ok'] for x in results) else 1)
