import os,json,subprocess,pathlib,hashlib
run=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/addfields-check');binary=pathlib.Path('/home/gsc/code/glassmind/target/release/glassmind')
os.environ.update(TMPDIR=str(run),CARGO_TARGET_DIR=str(run/'scalar-target'),GLASSMIND_SCRATCH=str(run/'scalar-scratch'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4')
rows=[]
for name,ty,value,nl in [('signed_unicode','i16','-9','\r\n'),('boolean','bool','true','\n'),('unicode_char','char',"'é'",'\n')]:
 root=run/'scalars'/name;(root/'src').mkdir(parents=True)
 (root/'Cargo.toml').write_text('[package]\nname="scalar_probe"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
 raw=('/* Zürich 🦀 */ pub struct Vessel { pub old: u8 }'+nl+'fn main() { /* café */ let v = Vessel { old: 4 }; println!("{}", v.old); }'+nl).encode();source=root/'src/main.rs';source.write_bytes(raw)
 p=subprocess.run([str(binary),'do',str(root),f'Add pub flag: {ty} to Vessel. Use {value} for all existing constructors.'],capture_output=True,text=True,timeout=180)
 changed=source.read_bytes();decl=f' pub flag: {ty},'.encode();default=f' flag: {value},'.encode()
 assert changed.count(decl)==1 and changed.count(default)==1,(name,p.stdout)
 assert changed.replace(decl,b'').replace(default,b'')==raw,name
 with source.open('a') as f:f.write(f'\n#[test] fn requested_scalar() {{ let v=Vessel {{ old:4,flag:{value} }}; let _: {ty}=v.flag; assert_eq!(v.flag,{value}); assert_eq!(v.old,4); }}\n')
 cmd=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120','cargo','test','-j1','--','--test-threads','4']
 test=subprocess.run(cmd,cwd=root,capture_output=True,text=True,timeout=140);assert test.returncode==0,test.stderr
 rows.append(dict(case=name,passed=True,original_sha256=hashlib.sha256(raw).hexdigest(),edited_sha256=hashlib.sha256(changed).hexdigest(),stdout=p.stdout))
(run/'scalar-results.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows))
