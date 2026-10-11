"""Archive this bounded worker's approved source and local evidence only."""
import argparse, hashlib, json, shutil, subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);a=p.parse_args();f=a.run/'frozen-candidate';f.mkdir(exist_ok=True)
source=Path('/home/gsc/.cache/glassmind/continuous-20261010/extremum-source')
m=json.loads((source/'manifest.json').read_text())
for row in m['files']:
 data=(source/row['file']).read_bytes()
 assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'],row['file']
 dest=f/'train-source'/row['file'];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
shutil.copyfile(source/'manifest.json',f/'train-source/manifest.json')
assert len(m['files'])==110 and sum(x['bytes'] for x in m['files'])==57265
shutil.copytree('crates/world-rust',f/'final-source/crates/world-rust',dirs_exist_ok=True)
shutil.copytree('tools/coding-comparator',f/'final-source/tools/coding-comparator',dirs_exist_ok=True)
shutil.copyfile('tools/coding-repair-dev.py',f/'coding-repair-dev.py')
diff=subprocess.check_output(['git','diff','--binary','--','crates/world-rust'])
for root in ['crates/world-rust','tools/coding-comparator']:
 for path in subprocess.check_output(['git','ls-files','--others','--exclude-standard',root],text=True).splitlines():
  result=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',path],capture_output=True)
  assert result.returncode in (0,1);diff+=result.stdout
(f/'candidate.patch').write_bytes(diff)
records=[]
for path in sorted(f.rglob('*')):
 if path.is_file() and path.name!='freeze-manifest.json':
  data=path.read_bytes();records.append(dict(file=str(path.relative_to(f)),bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
(f/'freeze-manifest.json').write_text(json.dumps(dict(files=records,total_bytes=sum(x['bytes'] for x in records)),indent=2)+'\n')
print(len(records),'frozen files',sum(x['bytes'] for x in records),'bytes')
