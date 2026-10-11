"""Independent native replay of frozen source and exact comparator edit contracts."""
from pathlib import Path
import argparse,hashlib,json,os,re,signal,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
HERE=Path(__file__).resolve().parent
assert hashlib.sha256((HERE/'cases.json').read_bytes()).hexdigest()==json.loads((HERE/'freeze.json').read_text())['cases.json']
cases=json.loads((HERE/'cases.json').read_text());a.out=a.out.resolve();a.out.mkdir(exist_ok=False)
manifest='[package]\nname="probe"\nversion="0.1.0"\nedition="2024"\n[workspace]\n'
rows=[]
for c in cases:
    folder=a.out/c['name'];root=folder/'workspace';root.mkdir(parents=True)
    for name,text in c['files'].items():path=root/name;path.parent.mkdir(exist_ok=True);path.write_text(text)
    (root/'Cargo.toml').write_text(manifest)
    env={k:v for k,v in os.environ.items() if 'API_KEY' not in k}
    env.update(TMPDIR=str(folder/'tmp'),GLASSMIND_SCRATCH=str(folder/'scratch'),GLASSMIND_CARGO_TARGET=str(folder/'check-target'),GLASSMIND_CHECK_CACHE=str(folder/'check-cache'),CARGO_TARGET_DIR=str(folder/'native-target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4')
    Path(env['TMPDIR']).mkdir()
    def run(cmd,label):
        with (folder/(label+'.stdout')).open('w') as out,(folder/(label+'.stderr')).open('w') as err:
            proc=subprocess.Popen(cmd,cwd=root,env=env,stdout=out,stderr=err,start_new_session=True)
            try:rc=proc.wait(timeout=160)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid,signal.SIGTERM)
                try:proc.wait(timeout=10)
                except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
                rc=124
        return rc
    cargo=['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120','cargo','test','--offline','-j1','--','--test-threads','4']
    before=run(cargo,'original-test');logs=(folder/'original-test.stdout').read_text()+(folder/'original-test.stderr').read_text()
    assert before==101 and 'test result: FAILED' in logs and 'could not compile' not in logs,(c['name'],logs)
    action=run([str(a.binary.resolve()),'do',str(root),'--explain','Fix the failing tests.'],'action')
    after={name:(root/name).read_text() for name in c['files']};changed=after!=c['files']
    exact=not changed
    if c['kind']=='repair':
        name=c['file'];original=c['files'][name];assert original.count(c['old'])==1
        prefix,suffix=original.split(c['old']);actual=after[name]
        exact=actual.startswith(prefix) and actual.endswith(suffix) and ''.join(actual[len(prefix):len(actual)-len(suffix)].split()) in {''.join(v.split()) for v in c['new']}
        exact=exact and all(after[k]==v for k,v in c['files'].items() if k!=name)
    native=run(cargo,'after-test') if changed else None
    row=dict(name=c['name'],kind=c['kind'],original_compiled_and_failed=True,action_exit=action,changed=changed,exact_source=exact,native_exit=native,manifest_unchanged=(root/'Cargo.toml').read_text()==manifest,after=after)
    rows.append(row);(a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows),ensure_ascii=False,indent=2)+'\n')
    print(c['name'],'changed',changed,'exact',exact,'native',native,flush=True)
summary=dict(total=len(rows),originals_compiled_and_failed=len(rows),repair_total=sum(r['kind']=='repair' for r in rows),exact_repairs=sum(r['kind']=='repair' and r['exact_source'] and r['native_exit']==0 for r in rows),negative_edits=[r['name'] for r in rows if r['kind']=='unchanged' and r['changed']],unresolved_unchanged=all(not r['changed'] for r in rows if r['kind']=='unresolved'),timeouts=[r['name'] for r in rows if r['action_exit']==124],manifests_unchanged=all(r['manifest_unchanged'] for r in rows))
(a.out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary),flush=True)
