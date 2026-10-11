import json, os, subprocess
from pathlib import Path
RUN=Path(os.environ['TMPDIR']).parent
BASE=RUN/'baseline/glassmind'
SCOPE=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
os.environ['GLASSMIND_SCRATCH']=str(RUN/'scratch')
rows=[]
for i, request in enumerate(['where is function implement?', 'where is function caller?', 'where is function renamed?', 'where is function called?', 'where is function it?', 'rename caller to implement_again', 'rename implement to renamed', 'rename function implement to calculate', 'rename function caller to called', 'rename function caller to `called`', 'where is function `it`?', 'rename function it to calculate', 'rename function `it` to calculate', 'rename field called of struct Ledger to calculate', 'rename field `called` of struct Ledger to calculate', 'where is field called of struct Ledger?', 'where is field `called` of struct Ledger?']):
 root=RUN/'discovery'/str(i); (root/'src').mkdir(parents=True,exist_ok=True); (root/'tests').mkdir(exist_ok=True)
 (root/'Cargo.toml').write_text('[package]\nname="roles"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
 source='pub fn implement() -> u32 { 7 }\npub fn caller() -> u32 { implement() }\npub fn renamed() {}\npub fn called() {}\npub fn it() {}\npub struct Ledger { pub called: u32 }\npub fn measure() -> u32 { Ledger { called: 5 }.called }\n'
 (root/'src/lib.rs').write_text(source)
 (root/'tests/behavior.rs').write_text('#[test]\nfn preserved() { if roles::caller() != 7 || roles::measure() != 5 { std::process::exit(1); } }\n')
 p=subprocess.run(SCOPE+[str(BASE),'do',str(root),request],text=True,capture_output=True)
 rows.append(dict(request=request,stdout=p.stdout,stderr=p.stderr,changed=(root/'src/lib.rs').read_text()!=source))
 print(request, p.stdout.strip(),flush=True)
Path('tools/coding-identifier-roles/reports/discovery.json').write_text(json.dumps(rows,indent=2)+'\n')
