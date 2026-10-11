from pathlib import Path
import argparse,hashlib,json,shutil,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
inputs=Path('/home/gsc/.cache/glassmind/continuous-20261010/renameenglish-run/dev-inputs');provenance=json.loads((inputs/'provenance.json').read_text());rows=[]
for req in provenance['requests']:
 name=req['id'].split('/')[-1].split('#')[0]
 for mode in ['ask','do']:
  case=a.out/(name+'-'+mode);case.mkdir();root=case/'workspace';shutil.copytree(inputs/'workspace',root)
  before={n:(root/n).read_bytes() for n in provenance['files']}
  for n,m in provenance['files'].items():assert hashlib.sha256(before[n]).hexdigest()==m['sha256']
  expected={n:b.replace(b'parse_line',b'parse_entry') for n,b in before.items()};assert sum(v.count(b'parse_line') for v in before.values())==9
  def run(cmd,label):
   r=subprocess.run(cmd,cwd=root,capture_output=True,text=True,timeout=150);(case/(label+'.stdout')).write_text(r.stdout);(case/(label+'.stderr')).write_text(r.stderr);return r
  oldtest=run(['cargo','test','--offline','-j1','--','--test-threads','4'],'before-test');assert oldtest.returncode==0
  oldout=run(['cargo','run','--offline','-q','-j1'],'before-run');assert oldout.returncode==0
  preview=run([str(a.binary.resolve()),'ask','--source',str(root.resolve()),'--json',req['text']],'preview');j=json.loads(preview.stdout);assert j['response']['type']=='edit' and len(j['response']['changes'])==9
  assert all((root/n).read_bytes()==v for n,v in before.items())
  cmd=[str(a.binary.resolve()),'ask','--source',str(root.resolve()),'--json','--apply'] if mode=='ask' else [str(a.binary.resolve()),'do',str(root.resolve()),'--explain']
  result=run(cmd+[req['text']],'apply');assert result.returncode==0 and all((root/n).read_bytes()==v for n,v in expected.items()),(name,mode,result.stdout)
  newtest=run(['cargo','test','--offline','-j1','--','--test-threads','4'],'after-test');assert newtest.returncode==0
  newout=run(['cargo','run','--offline','-q','-j1'],'after-run');assert newout.returncode==0 and newout.stdout==oldout.stdout
  rows.append(dict(style=name,mode=mode,preview_readonly=True,exact_edits=9,native_before_after=True,binary_output_identical=True));print(name,mode,'passed',flush=True)
(a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows),indent=2)+'\n')
