"""Supplemental, post-implementation owner/source-ambiguity audit (not frozen gain)."""
import json,os,subprocess
from pathlib import Path
run=Path(os.environ['TMPDIR']).parent
scope=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
source='pub struct Change { pub called: u32 }\npub struct Ledger { pub called: u32 }\npub fn observe() -> u32 { Change { called: 3 }.called + Ledger { called: 4 }.called }\n// called stays here\npub const NOTE: &str = "called";\n'
tests='#[test] fn behavior() { assert_eq!(roles::observe(), 7); }\n'
expected=source.replace('Change { pub called','Change { pub saved').replace('Change { called: 3 }.called','Change { saved: 3 }.saved')
cases=[('query-owner','where is field called of struct Change?',source,'`Change::called` is defined at src/lib.rs:1.\n'),('query-other','where is field called of struct Ledger?',source,'`Ledger::called` is defined at src/lib.rs:2.\n'),('rename-owner','rename field called on Change to saved',expected,None),('rename-control','rename field `called` on `Change` to `saved`',expected,None),('ambiguous','rename field `called` to `saved`',source,None),('negated','do not rename field called on Change to saved',source,None),('suffix','rename field called on Change to saved mysterious_suffix',source,None),('second-command','rename field called on Change to saved; rename field called on Ledger to updated',source,None)]
rows=[]
for mode,binary in [('baseline',run/'baseline/glassmind'),('candidate',run/'glassmind-candidate-v2')]:
 for name,request,wanted,reply in cases:
  root=run/'owner-audit'/mode/name;(root/'src').mkdir(parents=True);(root/'tests').mkdir()
  (root/'Cargo.toml').write_text('[package]\nname="roles"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
  (root/'src/lib.rs').write_text(source);(root/'tests/behavior.rs').write_text(tests)
  p=subprocess.run(scope+[str(binary),'do',str(root),request],text=True,capture_output=True)
  actual=(root/'src/lib.rs').read_text()
  native=subprocess.run(scope+['cargo','test','-j1','--','--test-threads','4'],cwd=root,text=True,capture_output=True) if actual!=source else None
  ok=actual==wanted and (reply is None or p.stdout==reply) and (actual==source or (native.returncode==0 and '1 passed' in native.stdout)) and (root/'tests/behavior.rs').read_text()==tests
  rows.append(dict(mode=mode,id=name,request=request,ok=ok,changed=actual!=source,stdout=p.stdout,stderr=p.stderr,actual=actual,native=None if native is None else dict(code=native.returncode,stdout=native.stdout,stderr=native.stderr)))
  print(mode,name,ok,repr(p.stdout),flush=True)
Path('tools/coding-identifier-roles/reports/owner-audit.json').write_text(json.dumps(rows,indent=2)+'\n')
