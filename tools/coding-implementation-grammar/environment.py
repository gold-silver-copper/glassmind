"""Supplemental environment/atomicity gates, expectations fixed before execution."""
import pathlib,os,json,subprocess,hashlib,shutil,sys
R=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run');T=pathlib.Path(__file__).resolve().parent;out=R/(sys.argv[2] if len(sys.argv)>2 else 'environment');out.mkdir(exist_ok=False);binary=R/(sys.argv[1] if len(sys.argv)>1 else 'glassmind-candidate');native=json.loads((T/'native.json').read_text());base=native[0]['code']
cases=[('set',True,base),('unset-target',True,base),('default-scratch',True,base),('live-scratch',False,base),('live-target',False,base),('live-tmp',False,base),('artifacts-refusal',False,native[8]['code']),('missing-cargo',False,base),('timeout',False,base+'#[cfg(test)] mod slow { #[test] fn wait() { std::thread::sleep(std::time::Duration::from_secs(20)); } }'),('unexecuted-failing',False,'pub fn collect_values(xs: &[i32]) -> i32 { todo!() }\n#[cfg(test)] mod tests { #[test] fn unrelated() { panic!("unrelated"); } }')]
(T/'reports/environment-definition.json').write_text(json.dumps(cases,indent=2))
results=[]
for name,accept,code in cases:
 root=out/name;(root/'src').mkdir(parents=True);(root/'src/lib.rs').write_text(code);(root/'Cargo.toml').write_text('[package]\nname="grammar_contract"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
 # Keep genuine existing artifacts through every accepted/refused request.
 shutil.copy(R/'baseline-native/sum/Cargo.lock',root/'Cargo.lock');(root/'target').mkdir();(root/'target/sentinel').write_text('existing opaque artifact\n')
 env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
 if name in ['unset-target','default-scratch']:env.pop('CARGO_TARGET_DIR',None)
 if name=='default-scratch':
  env.pop('GLASSMIND_SCRATCH',None);env['HOME']=str(out/'home');env['RUSTUP_HOME']='/home/gsc/.rustup';env['CARGO_HOME']='/home/gsc/.cargo'
 if name=='live-scratch':env['GLASSMIND_SCRATCH']=str(root/'scratch')
 if name=='live-target':env['CARGO_TARGET_DIR']=str(root/'cache')
 if name=='live-tmp':env['TMPDIR']=str(root/'tmp')
 if name=='missing-cargo':
  bins=out/'bin';bins.mkdir()
  for tool in ['systemd-run','setsid','timeout']:(bins/tool).symlink_to(shutil.which(tool))
  env['PATH']=str(bins)
 def snap():return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()}
 before=snap();p=subprocess.run([str(binary),'do',str(root),'implement collect_values'],env=env,capture_output=True,text=True,timeout=110);(out/(name+'.log')).write_text(p.stdout+p.stderr);after=snap();accepted='Applied test-verified proposal' in p.stdout
 ok=accepted==accept and (before==after if not accepted else set(before)==set(after) and all(before[k]==after[k] for k in before if k!='src/lib.rs'))
 results.append(dict(case=name,ok=ok,accepted=accepted,expected=accept,before=before,after=after,stdout=p.stdout));(out/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(name,ok,flush=True)
assert all(x['ok'] for x in results)
