#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Owned frozen development replay; never reads benchmark/hidden evaluation data."""
import argparse, hashlib, json, os, shutil, subprocess
from pathlib import Path
from cases import POSITIVES, NEGATIVES, behavior

def sha(b): return hashlib.sha256(b).hexdigest()
def run(cmd, cwd, env):
    p=subprocess.run(list(map(str,cmd)),cwd=cwd,env=env,capture_output=True,text=True,timeout=620)
    return dict(command=list(map(str,cmd)),exit=p.returncode,stdout=p.stdout,stderr=p.stderr)
def cargo(root,env,sub):
    args=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo',sub,'-j1']
    if sub=='test': args+=['--','--test-threads','4']
    return run(args,root,env)
def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--dev-inputs',type=Path,required=True);p.add_argument('--sample',action='store_true');p.add_argument('--baseline',action='store_true');a=p.parse_args()
    out=a.out.resolve();out.mkdir(parents=True,exist_ok=False)
    env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(a.dev_inputs.resolve().parent/'scratch')
    env.pop('GLASSMIND_CHECK_CACHE',None)
    env['GLASSMIND_CARGO_TARGET']=env['CARGO_TARGET_DIR']
    assert env['TMPDIR'] and not env['TMPDIR'].startswith('/tmp')
    supplied=(a.dev_inputs/'workspace/src/lib.rs').read_text()
    requests=json.loads((a.dev_inputs/'requests.json').read_text())
    supplied_test='''#[test] fn independent_contract() {
 let data = ["Ada", "Bea", "Cy"];
 for n in [i32::MIN, -2, -1, 0, 1, 2, 3, 4, i32::MAX] {
  let expected = match n { 1 => Some("Ada".to_string()), 2 => Some("Bea".to_string()), 3 => Some("Cy".to_string()), _ => None };
  assert_eq!(task::player_at_rank(&data,n),expected);
  assert_eq!(task::player_at_rank(&[],n),None);
 }
}'''
    cases=[('supplied_'+r['id'].split('/')[-1].split('#')[0],supplied,supplied.replace('players[index]','players[(index) as usize]'),supplied_test,r['text']) for r in requests if not a.sample or 'terse' in r['id']]
    positives=POSITIVES if not a.sample else [x for x in POSITIVES if x[0]=='zero_i16']
    negatives=NEGATIVES if not a.sample else [x for x in NEGATIVES if x[0]=='no_lower']
    for n,s,t,b,f in positives:
        index='rank' if n=='zero_i32' else 'index'
        expected=s.replace('['+index+']','[('+index+') as usize]')
        cases.append((n,s,expected,behavior(t,b,f),'fix compile error'))
    cases += [(n,s,None,None,'fix compile error') for n,s in negatives]
    # Freeze expected artifacts for EVERY case before invoking the solver.
    for n,s,expected,test,request in cases:
        d=out/n;d.mkdir();(d/'before.rs').write_text(s)
        if expected is not None: (d/'expected.rs').write_text(expected)
        if test: (d/'behavior.rs').write_text(test)
    rows=[]
    for n,s,expected,test,request in cases:
        d=out/n;root=d/'package';(root/'src').mkdir(parents=True)
        if n.startswith('supplied'): shutil.copytree(a.dev_inputs/'workspace',root,dirs_exist_ok=True)
        else: (root/'Cargo.toml').write_text('[package]\nname="task"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
        (root/'src/lib.rs').write_text(s)
        baseline=cargo(root,env,'check');(d/'original-diagnostic.json').write_text(json.dumps(baseline,indent=2))
        originals={str(p.relative_to(root)):p.read_bytes() for p in root.rglob('*') if p.is_file()}
        result=run([a.binary.resolve(),'do',root,request,'--explain'],root,env)
        after=(root/'src/lib.rs').read_text();(d/'after.rs').write_text(after)
        unchanged=all((root/path).read_bytes()==data for path,data in originals.items() if path!='src/lib.rs')
        changed=after!=s
        exact=after==(expected if expected is not None else s)
        native=None
        if changed and expected is not None:
            (root/'tests').mkdir(exist_ok=True);(root/'tests/independent.rs').write_text(test)
            native=cargo(root,env,'test');(d/'native.json').write_text(json.dumps(native,indent=2))
        row=dict(name=n,positive=expected is not None,changed=changed,exact=exact,unrelated_bytes_preserved=unchanged,original_e0277=baseline['exit']!=0 and 'E0277' in baseline['stderr'],native_pass=native is not None and native['exit']==0,result=result,before_sha=sha(s.encode()),after_sha=sha(after.encode()))
        row['passed']=(not changed if a.baseline else (exact and unchanged and (expected is None or (row['original_e0277'] and row['native_pass']))))
        # Supplied noncanonical requests remain outside the owned English grammar.
        if n in ('supplied_chatty','supplied_hurried') and not changed: row['passed']=True
        rows.append(row);(d/'result.json').write_text(json.dumps(row,indent=2));print(n, 'changed' if changed else 'unchanged', 'PASS' if row['passed'] else 'FAIL',flush=True)
    summary=dict(total=len(rows),passed=sum(x['passed'] for x in rows),positive_edits=sum(x['positive'] and x['changed'] and x['exact'] and x['native_pass'] for x in rows),unsafe_negative_edits=sum(not x['positive'] and x['changed'] for x in rows),binary_sha=sha(a.binary.read_bytes()),rows=rows)
    (out/'results.json').write_text(json.dumps(summary,indent=2));print({k:v for k,v in summary.items() if k!='rows'})
    raise SystemExit(0 if summary['passed']==len(rows) else 1)
if __name__=='__main__':main()
