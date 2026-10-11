#!/usr/bin/env python3
"""Visible AddField DEV replay and independently authored MIT/Apache-2.0 probes."""
import argparse, hashlib, json, os, re, shutil, subprocess
from pathlib import Path

ENV = {k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
assert ENV.get('TMPDIR') and not ENV['TMPDIR'].startswith('/tmp')
assert ENV.get('CARGO_TARGET_DIR')

def run(cmd, cwd=None):
    p = subprocess.run(list(map(str, cmd)), cwd=cwd, env=ENV, text=True, capture_output=True, timeout=650)
    return {'code':p.returncode, 'stdout':p.stdout, 'stderr':p.stderr}

def cargo(root, action):
    return run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo', action,'-j1'] + (['--','--test-threads','4'] if action=='test' else []), root)

def snapshot(root):
    return {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file() and p.name!='Cargo.lock'}

def exercise(binary, root, text, expected, baseline=False):
    before = snapshot(root)
    lock_before = (root/"Cargo.lock").read_bytes() if (root/"Cargo.lock").exists() else None
    behavior = cargo(root, 'run') if expected else None
    # Cargo.lock is generated before the edit; it is not a source change.
    result = run([binary,'do',root,text])
    after = snapshot(root)
    changed = before != after
    row = dict(request=text, result=result, changed=changed)
    if changed:
        owner, field, ty, default = expected
        edits = []
        # Independent source invariant: removing ONLY inserted declaration/value
        # text recovers every original byte, including assertions/comments/strings.
        declaration = f' pub {field}: {ty},'.encode()
        value = f' {field}: {default},'.encode()
        preserved = set(before)==set(after)
        for name, old in before.items():
            new = after[name]
            if new != old:
                edits.append(name)
                preserved &= new.replace(declaration,b'').replace(value,b'') == old
        joined = b''.join(after.values())
        structural = joined.count(declaration)==1 and joined.count(value)>=1
        # Add independent contract checks only AFTER byte-preservation checks.
        # Original public inline tests are left intact.
        if owner == 'Parcel':
            contract = '#[test] fn added_value() { let p=independent::Parcel::new(); let _: u16=p.weight; assert_eq!(p.weight,7); assert_eq!(p.old,9); }'
        elif owner == 'Book':
            contract = '#[test] fn added_value() { let books=task::seed::starter_books(); assert_eq!(books.len(),3); for b in books { let _:u32=b.pages; assert_eq!(b.pages,0); } }'
        else:
            contract = '#[test] fn added_value() { let p=task::Task::new("new"); let _:u8=p.priority; assert_eq!(p.priority,3); assert!(!p.done); for t in task::templates::moving_day() { assert_eq!(t.priority,3); assert!(!t.done); } let t=task::templates::example_done(); assert_eq!(t.priority,3); assert!(t.done); }'
        (root/'tests').mkdir(exist_ok=True)
        (root/'tests/addfield_contract.rs').write_text(contract)
        tests = cargo(root, 'test')
        runtime = cargo(root, 'run')
        row.update(files=edits, preserved=preserved, structural=structural, tests=tests, runtime=runtime,
                   behavior_preserved=behavior['code']==0 and runtime['code']==0 and behavior['stdout']==runtime['stdout'])
        row['passed'] = structural and preserved and tests['code']==0 and row['behavior_preserved']
    else:
        lock_after = (root/'Cargo.lock').read_bytes() if (root/'Cargo.lock').exists() else None
        row['exact_rollback'] = expected is not None or lock_before == lock_after
        row['passed'] = (expected is None or baseline) and row['exact_rollback']
    return row

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--binary',type=Path,required=True); ap.add_argument('--output',type=Path,required=True)
    ap.add_argument('--dev-inputs',type=Path); ap.add_argument('--sample',action='store_true'); ap.add_argument('--baseline',action='store_true'); ap.add_argument('--independent',action='store_true'); ap.add_argument('--cases', nargs='*')
    a=ap.parse_args(); a.output.mkdir(parents=True,exist_ok=False); rows=[]
    if a.dev_inputs:
        provenance=json.loads((a.dev_inputs/'provenance.json').read_text())
        for task in provenance['tasks']:
            name=task['task'].split('/')[-1]; source=a.dev_inputs/name
            for p in json.loads((source/'prompts.json').read_text()):
                if a.sample and p['style']!='terse': continue
                root=a.output/(name+'-'+p['style']); shutil.copytree(source/'workspace',root)
                # Expectations identify declared DEV task contracts, not runtime lookup tables.
                expected=('Book','pages','u32','0') if name=='add-book-pages' else ('Task','priority','u8','3')
                row=exercise(a.binary.resolve(),root,p['text'],expected,a.baseline); row['case']=root.name; rows.append(row)
    if a.independent:
        base='pub struct Parcel { pub old: u8 }\nimpl Parcel { pub fn new() -> Self { Self { old: 9 } } }\n'
        main='use independent::Parcel as Packet; fn main() { let p=Packet { old: 2 }; println!("{} {}",p.old, Packet::new().old); }'
        request='Add pub weight: u16 to Parcel. Use 7 for all existing constructors.'
        cases=[('alias-self',base,main,request,True),
          ('unrelated',base+'pub struct Other { pub weight: bool }\n',main,request,True),
          ('default', '#[derive(Default)]\n'+base,main,request,False),
          ('collision',base+'mod inner { pub struct Parcel { pub old: u8 } }',main,request,False),
          ('alias',base+'type Alias = Parcel;',main,request,False),
          ('unknown-macro',base+'macro_rules! make { () => { Parcel { old: 3 } }; }',main,request,False),
          ('update',base,main.replace('Packet { old: 2 }','Packet { ..Packet::new() }'),request,False),
          ('cfg',base.replace('pub struct','#[cfg(feature="x")] pub struct',1),main,request,False),
          ('trailing',base,main,request+' Also delete the tests.',False),
          ('unsafe-default',base,main,request.replace('Use 7','Use unsafe { std::mem::zeroed() }'),False),
          ('live-rollback',base+'#[test] fn check_location() { assert!(!std::env::current_dir().unwrap().ends_with("live-rollback")); }',main,request,False),
          ('unsafe-function',base+'unsafe fn arbitrary() -> Parcel { std::mem::zeroed() }',main,request,False),
          ('test-fails',base+'#[test] fn remains_failing() { assert_eq!(1, 2); }',main,request,False),
          ('type-mismatch',base,main,request.replace('Use 7','Use 999999'),False),
          ('local-shadow',base,main.replace('let p=', 'struct Packet { old: u8 } let p='),request,False)]
        for name,lib,main,text,positive in cases:
            if a.cases and name not in a.cases: continue
            root=a.output/name; (root/'src').mkdir(parents=True)
            (root/'Cargo.toml').write_text('[package]\nname="independent"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
            (root/'src/lib.rs').write_text(lib); (root/'src/main.rs').write_text(main)
            row=exercise(a.binary.resolve(),root,text,('Parcel','weight','u16','7') if positive else None,a.baseline); row['case']=name; rows.append(row)
    (a.output/'results.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(json.dumps({'cases':len(rows),'changed':sum(r['changed'] for r in rows),'passed':sum(r['passed'] for r in rows),'results':str(a.output/'results.json')}))
    if not all(r['passed'] for r in rows): raise SystemExit(1)
if __name__=='__main__': main()
