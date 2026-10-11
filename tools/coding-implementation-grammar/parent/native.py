from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--baseline',action='store_true');p.add_argument('--unset-target',action='store_true');a=p.parse_args()
HERE=Path(__file__).resolve().parent
for n,sha in json.loads((HERE/'freeze.json').read_text()).items():assert hashlib.sha256((HERE/n).read_bytes()).hexdigest()==sha,n
cases=json.loads((HERE/'cases.json').read_text());a.out.mkdir(exist_ok=False)
manifest='[package]\nname="native_probe"\nversion="0.1.0"\nedition="2021"\n[workspace]\n'
rows=[]
for c in cases:
    folder=a.out/c['name'];root=folder/'workspace';(root/'src').mkdir(parents=True)
    (root/'Cargo.toml').write_text(manifest);(root/'src/lib.rs').write_text(c['source'])
    original=folder/'original';shutil.copytree(root,original)
    env={k:v for k,v in os.environ.items() if 'API_KEY' not in k};env.update(TMPDIR=str(folder/'tmp'),GLASSMIND_SCRATCH=str(folder/'scratch'),GLASSMIND_CARGO_TARGET=str(folder/'checks'),GLASSMIND_CHECK_CACHE=str(folder/'cache'),CARGO_TARGET_DIR=str(folder/'target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4');Path(env['TMPDIR']).mkdir()
    def run(cmd,cwd,label):
        r=subprocess.run(cmd,cwd=cwd,env=env,text=True,capture_output=True,timeout=180)
        (folder/(label+'.stdout')).write_text(r.stdout);(folder/(label+'.stderr')).write_text(r.stderr);return r
    cargo=['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120','cargo','test','--offline','-j1','--','--test-threads','4']
    r=run(cargo,original,'original');logs=r.stdout+r.stderr
    if c['name'] in ['unexecuted','no_tests']:assert r.returncode==0,(c,logs)
    elif c['name']=='compiler_failure':assert r.returncode==101 and 'could not compile' in logs,(c,logs)
    else:assert r.returncode==101 and 'test result: FAILED' in logs and 'could not compile' not in logs,(c,logs)
    def snap():return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()}
    before=snap();saved_target=env.pop('CARGO_TARGET_DIR') if a.unset_target else None
    action=run([str(a.binary.resolve()),'do',str(root),c['request']],root,'action')
    if saved_target is not None:env['CARGO_TARGET_DIR']=saved_target
    after=snap();source=(root/'src/lib.rs').read_text();changed=before!=after
    accepted=False
    if c['kind']=='implement' and not a.baseline:
        stub='unimplemented!()' if 'unimplemented!()' in c['source'] else 'todo!()';left,right=c['source'].split(stub)
        # Proposal replaces whole stub body; allow only its original braced interior.
        prefix=left[:left.rfind('{')+1];suffix=right[right.index('}'):]
        exact=source.startswith(prefix) and source.endswith(suffix) and source!=c['source']
        native=run(cargo,root,'patched') if changed else None
        accepted=bool(exact and native and native.returncode==0)
        assert accepted,(c['name'],action.stdout,action.stderr,source)
        assert (root/'Cargo.toml').read_text()==manifest
        assert set(after)==set(before),(c['name'],set(after)-set(before))
    else:assert not changed,(c['name'],before,after,action.stdout)
    rows.append(dict(name=c['name'],kind=c['kind'],original_exit=r.returncode,changed=changed,accepted=accepted,action_exit=action.returncode,before=before,after=after,source=source))
    (a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),baseline=a.baseline,cases=rows),ensure_ascii=False,indent=2)+'\n')
    print(c['name'],'baseline',r.returncode,'changed',changed,'accepted',accepted,flush=True)
print(json.dumps(dict(total=len(rows),implementations=sum(x['accepted'] for x in rows),all_refusals_unchanged=all(not x['changed'] for x in rows if x['kind']!='implement'))))
