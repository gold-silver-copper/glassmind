#!/usr/bin/env python3
"""Independent authored route contracts frozen before runtime edits."""
import argparse, pathlib, json, os, subprocess, hashlib
SOURCE='''pub fn implement() -> i32 { 1 }
pub fn implemented() -> i32 { implement() }
pub fn caller() -> i32 { implemented() }
pub struct Ledger { pub implement: i32 }
pub fn ledger() -> Ledger { Ledger { implement: 3 } }
#[cfg(test)] mod tests { use super::*; #[test] fn check() { assert_eq!(caller(),1); assert_eq!(ledger().implement,3); } }
'''
# Disposition is implementation parser None/Err/Ok, separate from CLI success.
CASES=[
 ('query-active','which functions call implement?', 'none','query','implemented'),
 ('query-passive','which functions call implemented?', 'none','query','caller'),
 ('rename-active','rename implement to calculate','none','rename','calculate'),
 ('rename-passive','rename implemented to compute','none','rename','compute'),
 ('rename-to-active','rename caller to implement_again','none','rename','implement_again'),
 ('field-type','what is the type of Ledger.implement?','none','query','i32'),
 ('field-add','add public field implemented: i32 to Ledger with default 0','none','field','implemented'),
 ('negative','do not implement absent','refused','refuse',''),
 ('negative-polite',"please don't implement absent",'refused','refuse',''),
 ('negative-modal','could you not implement absent?','refused','refuse',''),
 ('negative-write','do not write the body of absent','refused','refuse',''),
 ('negative-fill',"don't fill in absent",'refused','refuse',''),
 ('passive-negative','absent should not be implemented','refused','refuse',''),
 ('multi-rename','rename implement to renamed then implement absent','refused','refuse',''),
 ('multi-field','add public field flag: i32 to Ledger with default 0 then implement absent','refused','refuse',''),
 ('multi-repair','repair caller and implement absent','refused','refuse',''),
 ('behavior','implement absent with wrapping arithmetic','refused','refuse',''),
 ('unknown','implement absent','absent','refuse',''),
 ('repair','repair caller','none','refuse',''),
 ('ordinary-word','which functions implemented this behavior?','none','refuse',''),
 ('quoted','Please implement "語".','語','refuse',''),
 ('stub','absent is a stub. Please implement it.','absent','refuse',''),
 ('ambiguous','absent and other are stubs, implement it','refused','refuse',''),
]
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--out',required=True);p.add_argument('--probe');a=p.parse_args()
 out=pathlib.Path(a.out).resolve();out.mkdir(parents=True,exist_ok=False);rows=[]
 for unset in [False,True]:
  for name,text,disposition,kind,value in CASES:
   root=out/(name+('-unset' if unset else '-set'));(root/'src').mkdir(parents=True)
   (root/'Cargo.toml').write_text('[package]\nname="route_contract"\nversion="0.1.0"\nedition="2021"\n');(root/'src/lib.rs').write_text(SOURCE)
   def snap():return {str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest() for f in root.rglob('*') if f.is_file()}
   before=snap();env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
   if unset:env.pop('TMPDIR',None);env.pop('CARGO_TARGET_DIR',None)
   r=subprocess.run([a.binary,'do',str(root),text],env=env,capture_output=True,text=True,timeout=110)
   (out/(root.name+'.log')).write_text(r.stdout+r.stderr);after=snap();code=(root/'src/lib.rs').read_text()
   if kind=='rename':ok=f'fn {value}(' in code and before!=after
   elif kind=='field':ok=f'pub {value}: i32' in code and before!=after
   elif kind=='query':ok=value in r.stdout and before==after and 'Cannot implement:' not in r.stdout
   else:ok=before==after and 'Applied test-verified' not in r.stdout
   parsed=None
   if a.probe:
    v=subprocess.run([a.probe],input=text+'\n',capture_output=True,text=True,check=True);parsed=v.stdout.strip();expected={'none':'UNRECOGNIZED','refused':'REFUSED'}.get(disposition,disposition);ok=ok and parsed==expected
   row=dict(case=root.name,text=text,expected_disposition=disposition,parsed=parsed,ok=ok,exit=r.returncode,before=before,after=after,stdout=r.stdout);rows.append(row);print(root.name,ok,flush=True)
 (out/'results.json').write_text(json.dumps(rows,indent=2,ensure_ascii=False)+'\n')
 raise SystemExit(0 if all(x['ok'] for x in rows) else 1)
