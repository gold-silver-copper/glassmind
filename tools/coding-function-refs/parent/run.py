from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
base=Path(__file__).parent
raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['cases_sha256']
cases=json.loads(raw);a.out.mkdir(parents=True,exist_ok=False);rows=[]
for case in cases:
 root=a.out/case['name'];root.mkdir()
 for name,value in case['files'].items():
  f=root/name;f.parent.mkdir(parents=True,exist_ok=True);f.write_bytes(value.encode())
 def run(args,label):
  r=subprocess.run(args,cwd=root,capture_output=True,text=True,timeout=150)
  (root/(label+'.stdout')).write_text(r.stdout);(root/(label+'.stderr')).write_text(r.stderr)
  return r
 before=run(['cargo','test','--offline','-j1','--','--test-threads','4'],'before');assert before.returncode==0,(case['name'],before.stderr)
 before_bin=None
 if 'src/main.rs' in case['files']:
  before_bin=run(['cargo','run','--offline','-q','-j1'],'before-bin');assert before_bin.returncode==0
 r=run([str(a.binary.resolve()),'do',str(root.resolve()),case['request']],'solver')
 after={name:(root/name).read_bytes() for name in case['files']}
 changed=any(after[k]!=v.encode() for k,v in case['files'].items())
 exact=not changed or case['expected'] is not None and all(after[k]==v.encode() for k,v in case['expected'].items())
 native=run(['cargo','test','--offline','-j1','--','--test-threads','4'],'after')
 output_same=True
 if before_bin:
  after_bin=run(['cargo','run','--offline','-q','-j1'],'after-bin');output_same=after_bin.returncode==0 and before_bin.stdout==after_bin.stdout
 row=dict(name=case['name'],changed=changed,required=case['required'],exact=exact,native_before=before.returncode,native_after=native.returncode,binary_output_preserved=output_same,solver_exit=r.returncode,reply=r.stdout,after_hashes={k:hashlib.sha256(v).hexdigest() for k,v in after.items()})
 rows.append(row);print(case['name'],'edited' if changed else 'unchanged','exact',exact,'native',native.returncode,flush=True)
 (a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows,edits=sum(x['changed'] for x in rows),unsafe_edits=sum(not x['exact'] or x['native_after']!=0 or not x['binary_output_preserved'] for x in rows),required_pass=sum(x['required'] and x['changed'] and x['exact'] for x in rows)),indent=2)+'\n')
