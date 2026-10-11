"""Audit new evidence, unchanged old files, and the public source manifest."""
import argparse,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--run',type=Path,required=True);r=p.parse_args().run
contracts=json.loads((r/'native-evidence/contracts.json').read_text())
assert len(contracts)==30
assert sum(c['choice'] is not None for c in contracts)==19
assert next(c for c in contracts if c['name']=='competing')['outputs']==['2 1\n','1 2\n']
for c in contracts:
 assert c['original_compiled']
 if c['choice'] is not None:assert c['original_failed'] and c['passing']==1
original=r/'original-source/world-rust';current=Path('crates/world-rust')
changed=[str(p.relative_to(original)) for p in original.rglob('*') if p.is_file() and p.read_bytes()!=(current/p.relative_to(original)).read_bytes()]
assert sorted(changed)==['src/checks.rs','src/lib.rs']
choices=[]
for p in sorted(r.glob('**/choice.json')):
 if not p.parent.name.startswith('comparator-'):continue
 c=json.loads(p.read_text())
 assert not c['accepted'] or not c['incomplete'] and len(c['passing'])==1
 choices.append(dict(file=str(p.relative_to(r)),accepted=c['accepted'],incomplete=c['incomplete'],passing=len(c['passing']),checks=len(c['native_checks'])))
src=Path('/home/gsc/.cache/glassmind/continuous-20261010/extremum-source');m=json.loads((src/'manifest.json').read_text())
for x in m['files']:
 b=(src/x['file']).read_bytes();assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256']
assert len(m['files'])==110 and sum(x['bytes'] for x in m['files'])==57265
report=dict(contracts=30,accepted=19,refused=11,changed_old_files=changed,choices=choices,source_files=110,source_bytes=57265,source_revision=m['revision'],license=m['license'])
(r/'audit.json').write_text(json.dumps(report,indent=2)+'\n')
print('audit passed:',len(choices),'native choices')
