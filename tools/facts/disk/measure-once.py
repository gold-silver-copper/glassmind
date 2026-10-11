"""Measure one command from a fresh small launcher, before loading large JSON."""
import os,subprocess,sys,time,resource,json,hashlib
from pathlib import Path
root=Path(__file__).resolve().parent
which=sys.argv[1];work=root.parents[1];bundle=work/'trainindex-run/tmp/trainindex/bundle1000'
if which=='baseline':cmd=[work/'linkquality-run/baseline/predictions',bundle/'relation.json',bundle/'index.json']
else:cmd=[root/'target/release/glassmind-facts','predict-disk','--model',bundle/'relation.json','--database',root/'native999.sqlite']
with Path(cmd[0]).open('rb') as stream: binary_hash=hashlib.file_digest(stream,'sha256').hexdigest()
parent=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
start=time.monotonic()
with (root/'baseline-1000.stdin').open('rb') as inp,(root/f'isolated-{which}.out').open('wb') as out,(root/f'isolated-{which}.err').open('wb') as err:
 child=subprocess.Popen(list(map(str,cmd)),stdin=inp,stdout=out,stderr=err)
 pid,status,usage=os.wait4(child.pid,0);child.returncode=os.waitstatus_to_exitcode(status)
 assert child.returncode==0
report={'binary_sha256':binary_hash,'mode':which,'rows':1000,'wall_seconds':round(time.monotonic()-start,6),'max_rss_kib':usage.ru_maxrss,'launcher_peak_rss_kib':parent,'note':'process peak includes fork/exec; launcher peak is below measured child peak'}
(root/f'isolated-{which}.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
