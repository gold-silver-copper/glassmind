"""Independent native adapter for frozen dedup-order cases; expected bytes stay fixed."""
from pathlib import Path
import argparse,hashlib,json,os,signal,subprocess,re
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();base=Path(__file__).parent;raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];cases=json.loads(raw);a.out=a.out.resolve();a.out.mkdir(parents=True);assert not (a.out/'report.json').exists();binary=a.binary.resolve();rows=[]
def exact(c,after):
 if c['kind']!='repair':return after==c['source']
 assert c['source'].count(c['old'])==1
 prefix,suffix=c['source'].split(c['old'])
 normalize=lambda value: re.sub(r'<(i(?:8|16|32|64|128|size))>::', r'\1::', ''.join(value.split()))
 return after.startswith(prefix) and after.endswith(suffix) and normalize(after[len(prefix):len(after)-len(suffix)]) in {normalize(value) for value in c['new']}
for c in cases:
 case=a.out/c['name'];root=case/'workspace';(root/'src').mkdir(parents=True);manifest='[package]\nname="parent_'+c['name']+'"\nversion="0.1.0"\nedition="2024"\n';(root/'Cargo.toml').write_text(manifest);(root/'src/lib.rs').write_text(c['source']);env={k:v for k,v in os.environ.items() if 'API_KEY' not in k};env.update(TMPDIR=str(case/'tmp'),GLASSMIND_SCRATCH=str(case/'scratch'),GLASSMIND_CARGO_TARGET=str(case/'check-target'),GLASSMIND_CHECK_CACHE=str(case/'check-cache'),CARGO_TARGET_DIR=str(case/'native-target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4');Path(env['TMPDIR']).mkdir()
 def run(cmd,label):
  with (case/(label+'.stdout')).open('w') as out,(case/(label+'.stderr')).open('w') as err:
   proc=subprocess.Popen(cmd,cwd=root,env=env,stdout=out,stderr=err,start_new_session=True)
   try:rc=proc.wait(timeout=160)
   except subprocess.TimeoutExpired:
    os.killpg(proc.pid,signal.SIGTERM)
    try:proc.wait(timeout=10)
    except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
    rc=124
  return rc
 cargo=['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120','cargo','test','--offline','-j1','--','--test-threads','4'];before=run(cargo,'original-test');original_log=(case/'original-test.stdout').read_text()+(case/'original-test.stderr').read_text();compiled_failed=before==101 and 'test result: FAILED' in original_log and 'could not compile' not in original_log
 assert compiled_failed,(c['name'],before,original_log)
 action=run([str(binary),'do',str(root),'--explain','Fix the failing tests.'],'action');after=(root/'src/lib.rs').read_text();changed=after!=c['source'];native=run(cargo,'after-test') if changed else None
 row=dict(name=c['name'],kind=c['kind'],original_compiled_and_failed=compiled_failed,original_test_exit=before,action_exit=action,changed=changed,exact_source=exact(c,after),manifest_unchanged=(root/'Cargo.toml').read_text()==manifest,native_exit=native,after_source=after)
 rows.append(row);(a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),cases=rows),ensure_ascii=False,indent=2)+'\n');print(c['name'],'changed',changed,'exact',row['exact_source'],'native',native,flush=True)
summary=dict(total=len(rows),originals_compiled_and_failed=sum(r['original_compiled_and_failed'] for r in rows),exact_repairs=sum(r['kind']=='repair' and r['exact_source'] and r['native_exit']==0 for r in rows),repair_total=sum(r['kind']=='repair' for r in rows),negative_edits=[r['name'] for r in rows if r['kind']=='unchanged' and r['changed']],unresolved_source_unchanged=all(not r['changed'] for r in rows if r['kind']=='unresolved'),unresolved_choice_evidence='Manual review of retained action/native evidence required; unchanged alone does not satisfy this gate',timeouts=[r['name'] for r in rows if r['action_exit']==124],manifest_unchanged=all(r['manifest_unchanged'] for r in rows));(a.out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary),flush=True)
