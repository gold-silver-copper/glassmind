import os,json,subprocess,shutil,sys,hashlib
from pathlib import Path
binary=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve(); out.mkdir(parents=True,exist_ok=True)
fixtures=json.loads((Path(__file__).parent/'fixtures.json').read_text()); results=[]
env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
env.update(GLASSMIND_SCRATCH=str(out/'scratch'),GLASSMIND_CHECK_CACHE=str(out/'checks'),GLASSMIND_CARGO_TARGET=str(Path(env['CARGO_TARGET_DIR'])/'authored'))
for f in fixtures:
 root=out/'sources'/f['name']; (root/'src').mkdir(parents=True,exist_ok=True)
 (root/'Cargo.toml').write_text('[package]\nname="'+f['name']+'"\nversion="0.0.0"\nedition="2024"\n[workspace]\n')
 code=f['code']+'#[cfg(test)] mod tests { use super::*; #[test] fn visible() { '+f['tests']+' } }\n'
 (root/'src/lib.rs').write_text(code)
 for p,t in f['extra'].items(): (root/p).write_text(t)
 before={str(p.relative_to(root)):p.read_bytes() for p in root.rglob('*') if p.is_file()}
 req=dict(id=f['name'],sources=[f['name']],turns=[dict(id=f['name']+'#0',text='Fix the failing tests.')])
 inp=out/'one.jsonl'; inp.write_text(json.dumps(req)+'\n')
 with (out/(f['name']+'.log')).open('w') as log:
  run=subprocess.run([str(binary),'eval','--sources',str(out/'sources'),'--inputs',str(inp),'--jobs','1'],env=env,text=True,stdout=subprocess.PIPE,stderr=log,check=True)
 result=json.loads(run.stdout); (out/(f['name']+'.json')).write_text(json.dumps(result,indent=2)+'\n')
 changes=result['response'].get('changes',[]); exact=False; native=None
 assert before=={str(p.relative_to(root)):p.read_bytes() for p in root.rglob('*') if p.is_file()}
 if changes:
  exact=len(changes)==1 and changes[0]['old']==f['old'] and changes[0]['new']==f['new'] and changes[0]['file']=='src/lib.rs'
  for c in sorted(changes,key=lambda c:(c['file'],c['line'],c['column']),reverse=True):
   p=root/c['file']; text=p.read_text(); offset=sum(map(len,text.splitlines(keepends=True)[:c['line']-1]))+c['column']; assert text[offset:offset+len(c['old'])]==c['old']; p.write_text(text[:offset]+c['new']+text[offset+len(c['old']):])
  with (out/(f['name']+'-native.log')).open('w') as log:
   native=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--test-threads','4'],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT).returncode==0
  expected=code.replace(f['old'],f['new'],1) if exact else ''
  exact=exact and (root/'src/lib.rs').read_text()==expected and all((root/p).read_bytes()==b for p,b in before.items() if p!='src/lib.rs')
 witness=list((out/'scratch').glob('ambiguity-*/witness.json'))
 ok=exact and native if f['kind'] in ('positive','legacy') else not changes
 if f['kind']=='ambiguity': ok=ok and any(json.loads(p.read_text())['original_source']==code for p in witness)
 row=dict(name=f['name'],kind=f['kind'],changes=changes,exact=exact,native=native,ok=ok)
 results.append(row); print(f['name'],ok,flush=True); (out/'report.json').write_text(json.dumps(results,indent=2)+'\n')
