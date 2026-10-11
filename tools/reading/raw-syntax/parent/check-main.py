from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
r=Path('/home/gsc/.cache/glassmind/continuous-20261010');candidate=r/'rawsyntax-run/candidate';baseline=r/'rawsyntax-run/baseline';ind=r/'coordinator/rawsyntax-independent/candidate';base=[str(a.binary.resolve()),'syntax','--model',str(candidate/'model.bin')];rows=[]
def run(name,cmd,check):
 x=subprocess.run(cmd,capture_output=True,text=True,timeout=90);(a.out/(name+'.stdout')).write_text(x.stdout);(a.out/(name+'.stderr')).write_text(x.stderr);ok=bool(check(x));rows.append(dict(name=name,exit=x.returncode,passed=ok));assert ok,(name,x.stdout[:1000],x.stderr[:1000]);return x
for name,inputs,expected in [('raw_full',candidate/'full-inputs.jsonl',candidate/'full-predictions.jsonl'),('forms_full',baseline/'forms.jsonl',baseline/'forms-predictions.jsonl'),('raw_sample',candidate/'sample-inputs.jsonl',candidate/'sample-predictions.jsonl'),('authored',candidate/'authored-inputs.jsonl',candidate/'authored-predictions.jsonl'),('contracts',candidate/'contracts-input.jsonl',candidate/'contracts-output.jsonl'),('independent',ind/'inputs.jsonl',ind/'predictions.jsonl')]:
 run(name,base+['--input',str(inputs)],lambda x:x.returncode==0 and x.stdout.encode()==expected.read_bytes())
run('plain_prediction',base+['Nora thanked Felix.'],lambda x:x.returncode==0 and x.stdout.startswith('Predicted dependencies (unverified):') and '"Nora" --nsubj--> 2: "thanked"' in x.stdout and '"Felix" --obj--> 2: "thanked"' in x.stdout)
def valid_json(x):
 if x.returncode:return False
 j=json.loads(x.stdout);return j['status']=='predicted' and j['tokenization']=='raw_unicode_apostrophe_v2' and j['forms']==['Nora','thanked','Felix','.'] and all(b'Nora thanked Felix.'[s['start']:s['end']].decode()==s['form'] for s in j['spans'])
run('single_json',base+['--json','Nora thanked Felix.'],valid_json)
for name,text in [('empty',''),('too_many',' '.join(['token']*129)),('too_long','x'*16385)]:run(name,base+['--json',text],lambda x:x.returncode==0 and json.loads(x.stdout)['status']=='refused')
large=a.out/'large.jsonl';large.write_bytes(b'x'*(128*1024+1));run('row_bound',base+['--input',str(large)],lambda x:x.returncode!=0 and not x.stdout and 'exceeds128KiB' in x.stderr)
invalid=a.out/'invalid.jsonl';invalid.write_bytes(b'\xff\n');run('invalid_utf8',base+['--input',str(invalid)],lambda x:x.returncode!=0 and not x.stdout)
run('conflicting_modes',base+['--input',str(ind/'inputs.jsonl'),'Nora thanked Felix.'],lambda x:x.returncode!=0 and not x.stdout)
run('missing_input',base,lambda x:x.returncode!=0 and not x.stdout)
run('corrupt_model',[str(a.binary.resolve()),'syntax','--model',str(candidate/'corrupt-model.bin'),'Nora thanked Felix.'],lambda x:x.returncode!=0 and not x.stdout)
run('help',[str(a.binary.resolve()),'syntax','--help'],lambda x:x.returncode==0 and 'not verified semantic claims' in x.stdout)
result=dict(total=len(rows),passed=sum(x['passed'] for x in rows),rows=rows,binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),raw_full_sha256=hashlib.sha256((a.out/'raw_full.stdout').read_bytes()).hexdigest(),forms_full_sha256=hashlib.sha256((a.out/'forms_full.stdout').read_bytes()).hexdigest(),scope='Main command output equivalence, raw marker deliberately v2; provided-form bytes unchanged. Input/error/provenance contracts, not semantic accuracy.')
(a.out/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
