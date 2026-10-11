"""Preserve terminal teacher artifacts without visiting credential contents."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
root=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/quantity-graphs');teacher=root/'teacher'
completion=json.loads((teacher/'completion.json').read_text())
process=json.loads((teacher/'process.json').read_text())
state=subprocess.run(['systemctl','--user','show',process['scope'],'-pActiveState','--value'],text=True,capture_output=True,timeout=20).stdout.strip()
assert state in ('inactive','failed',''),state
out=root/'teacher-originals';out.mkdir(exist_ok=False)
paths=[p for p in (teacher/'work').rglob('*') if not p.is_dir()]
paths += [teacher/n for n in ['completion.json','process.json','launch.json','isolation-check.json','login-status.txt','events.jsonl','stderr.log']]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=[]
for path in paths:
 assert not path.is_symlink() and path.is_file(),path
 relative=path.relative_to(teacher)
 assert 'codex' not in relative.parts and relative.name!='auth.json'
 target=out/relative;target.parent.mkdir(exist_ok=True,parents=True)
 shutil.copyfile(path,target)
 assert sha(path)==sha(target)
 target.chmod(0o444)
 manifest.append(dict(file=str(relative),bytes=target.stat().st_size,sha256=sha(target)))
(out/'manifest.json').write_text(json.dumps(dict(files=manifest,credential_contents_excluded=True),indent=2)+'\n')
turns=[]
for line in (out/'events.jsonl').read_text().splitlines():
 event=json.loads(line)
 if event.get('type')=='turn.completed':turns.append(event['usage'])
usage=dict(turns=len(turns),input_tokens=sum(t['input_tokens'] for t in turns),cached_input_tokens=sum(t.get('cached_input_tokens',0) for t in turns),output_tokens=sum(t['output_tokens'] for t in turns),cached_counted_once=True)
usage['total_counted']=usage['input_tokens']+usage['output_tokens']
(root/'teacher-usage.json').write_text(json.dumps(usage,indent=2)+'\n')
link=teacher/'codex/auth.json'
assert link.is_symlink() and link.readlink()==Path('/home/gsc/.codex/auth.json')
link.unlink()
cleanup=dict(scope_state=state,owned_login_symlink_removed=True,originals_hash_verified=True,credential_contents_read=False)
(root/'teacher-cleanup.json').write_text(json.dumps(cleanup,indent=2)+'\n')
print(json.dumps(dict(completion=completion,archived_files=len(manifest),archived_bytes=sum(x['bytes'] for x in manifest),usage=usage,cleanup=cleanup)))
