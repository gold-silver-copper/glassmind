from pathlib import Path
import argparse, hashlib, json, subprocess

p=argparse.ArgumentParser()
p.add_argument('--binary',type=Path,required=True)
p.add_argument('--artifacts',type=Path,required=True)
p.add_argument('--independent',type=Path,required=True)
p.add_argument('--out',type=Path,required=True)
a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
base=[str(a.binary.resolve()),'syntax','--model',str(a.artifacts/'model.bin')]
rows=[]
def run(name,cmd,check):
    r=subprocess.run(cmd,text=True,capture_output=True,timeout=90)
    (a.out/(name+'.stdout')).write_text(r.stdout)
    (a.out/(name+'.stderr')).write_text(r.stderr)
    ok=bool(check(r));rows.append(dict(name=name,exit=r.returncode,passed=ok))
    assert ok,(name,r.stdout[:800],r.stderr[:800])
    return r
for name in ['sample','full','authored','contracts']:
    expected=a.artifacts/(name+'-predictions.jsonl' if name!='contracts' else 'contracts-output.jsonl')
    run(name,base+['--input',str(a.artifacts/(name+'-input.jsonl'))],lambda r:r.returncode==0 and r.stdout.encode()==expected.read_bytes())
run('independent',base+['--input',str(a.independent/'inputs.jsonl')],lambda r:r.returncode==0 and r.stdout.encode()==(a.independent/'predictions.jsonl').read_bytes())
run('plain_prediction',base+['Nora thanked Felix.'],lambda r:r.returncode==0 and r.stdout.startswith('Predicted dependencies (unverified):') and '"Nora" --nsubj--> 2: "thanked"' in r.stdout and '"Felix" --obj--> 2: "thanked"' in r.stdout)
first=json.loads((a.independent/'predictions.jsonl').read_text().splitlines()[1])
run('single_json',base+['--json','Nora thanked Felix.'],lambda r:r.returncode==0 and json.loads(r.stdout)==first)
for name,text in [('empty',''),('too_many',' '.join(['token']*129)),('too_long','x'*16385)]:
    run(name,base+['--json',text],lambda r:r.returncode==0 and json.loads(r.stdout)['status']=='refused')
large=a.out/'large.jsonl';large.write_bytes(b'x'*(128*1024+1))
run('row_bound',base+['--input',str(large)],lambda r:r.returncode!=0 and not r.stdout and 'exceeds128KiB' in r.stderr)
invalid=a.out/'invalid.jsonl';invalid.write_bytes(b'\xff\n')
run('invalid_utf8',base+['--input',str(invalid)],lambda r:r.returncode!=0 and not r.stdout)
run('conflicting_modes',base+['--input',str(a.independent/'inputs.jsonl'),'Nora thanked Felix.'],lambda r:r.returncode!=0 and not r.stdout)
run('missing_input',base,lambda r:r.returncode!=0 and not r.stdout)
run('corrupt_model',[str(a.binary.resolve()),'syntax','--model',str(a.artifacts/'corrupt-model.bin'),'Nora thanked Felix.'],lambda r:r.returncode!=0 and not r.stdout)
run('help',[str(a.binary.resolve()),'syntax','--help'],lambda r:r.returncode==0 and 'not verified semantic claims' in r.stdout)
result=dict(total=len(rows),passed=sum(x['passed'] for x in rows),rows=rows,binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),full_prediction_sha256=hashlib.sha256((a.out/'full.stdout').read_bytes()).hexdigest(),scope='Main-command contracts and exact retained-output equivalence; linguistic errors are preserved.')
(a.out/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
