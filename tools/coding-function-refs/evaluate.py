#!/usr/bin/env python3
"""Public authored development cases; never imports evaluation definitions."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess
P=argparse.ArgumentParser(); P.add_argument('--binary',required=True); P.add_argument('--output',required=True); P.add_argument('--sample',action='store_true'); P.add_argument('--supplied'); a=P.parse_args()
out=pathlib.Path(a.output).resolve(); out.mkdir(parents=True,exist_ok=True)
binary=str(pathlib.Path(a.binary).resolve())
scope=['systemd-run','--user','--scope','--quiet','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
assert os.environ['TMPDIR'] and not os.environ['TMPDIR'].startswith('/tmp')
manifest="[package]\nname='proof-box'\nversion='0.0.0'\nedition='2024'\n[workspace]\n"
def positive(code, main=None):
 d={'Cargo.toml':manifest,'src/lib.rs':code,'notes.txt':'parse_line remains prose\n'}
 if main: d['src/main.rs']=main
 return d
base='pub fn parse_line(x: i32) -> i32 { x + 3 }\n'
test='#[test] fn checks() { assert_eq!(parse_line(4), 7); }\n'
cases=[]
def add(name,code,ok=True,main=None,old='parse_line',new='parse_entry'):
 cases.append((name,positive(code,main),ok,old,new))
add('grouped','mod parser { '+base+'pub const N:i32=4; } pub use parser::{parse_line,N};\n#[test] fn checks(){ assert_eq!(parse_line(N),7); }')
add('alias','mod parser { '+base+'} use parser::parse_line as decode;\n#[test] fn checks(){assert_eq!(decode(4),7);}')
add('super_glob',base+'#[cfg(test)] mod tests { use super::*; '+test+'}')
add('selflib',base+test,main='use proof_box::parse_line as decode; fn main(){assert_eq!(decode(4),7); println!("{}",decode(4));}')
add('callback',base+'#[test] fn checks(){let f:fn(i32)->i32=parse_line; assert_eq!([1,2].into_iter().map(f).sum::<i32>(),9); assert_eq!([1,2].into_iter().map(parse_line).sum::<i32>(),9);}')
add('qualified','pub mod parser {'+base+'} #[test] fn checks(){assert_eq!(crate::parser::parse_line(4),7);}')
add('unicode_crlf','pub fn café(x:i32)->i32{x+3}\r\n#[test] fn checks(){ let 文=4; assert_eq!(café(文),7); }\r\n',old='café',new='entrée')
add('shadow_local',base+test+'fn other(){let parse_line=4; assert_eq!(parse_line,4);}',False)
add('shadow_param',base+test+'fn other(parse_line:i32)->i32{parse_line}',False)
add('shadow_closure',base+test+'fn other(){let f=|parse_line:i32|parse_line; assert_eq!(f(2),2);}',False)
add('multiple',base+test+'mod other {'+base+'}',False)
add('field',base+test+'struct S {parse_line:i32} fn other(){let x=S{parse_line:2}; assert_eq!(x.parse_line,2);}',False)
add('method',base+test+'struct S; impl S{fn parse_line(&self)->i32{2}} fn other(){assert_eq!(S.parse_line(),2);}',False)
add('ambiguous_glob','mod p {'+base+'} mod q {'+base+'} use p::*; use q::*; '+test,False)
add('external_import',base+test+'use std::collections::BTreeMap;',False)
add('custom_macro',base+test+'macro_rules! custom {($e:expr)=>{$e}} fn other(){custom!(parse_line(4));}',False)
add('cfg', '#[cfg(not(any()))]\n'+base+test,False)
add('layout',base+test,False); cases[-1][1]['extra.rs']='fn unrelated(){}'
add('collision',base+test+'fn parse_entry(){}',False)
add('raw',base+test+'fn other(){r#parse_line(1);}',False)
add('case_comments',base+test+'// parse_line in a comment\n#[allow(non_snake_case)] fn Parse_Line(){}',False)
# Positive exact-case probe uses a type (no lint attribute needed).
add('case_literal',base+test+'pub struct ParseLine; pub const TEXT:&str="parse_line"; // parse_line\n')
add('block_function',base+test+'fn other(){fn parse_line()->i32{2} assert_eq!(parse_line(),2);}',False)
if a.sample: cases=[c for c in cases if c[0] in ['grouped','alias','selflib','shadow_local','custom_macro']]
if a.supplied:
 root=pathlib.Path(a.supplied); cases.append(('supplied',{str(p.relative_to(root)):p.read_text() for p in root.rglob('*') if p.is_file()},True,'parse_line','parse_entry'))
results=[]
def run(cmd,cwd,log):
 p=subprocess.run(cmd,cwd=cwd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=650); log.write_text(p.stdout); return p
for name,files,expected,old,new in cases:
 d=out/name; root=d/'workspace'; root.mkdir(parents=True,exist_ok=True)
 for f,s in files.items():
  p=root/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(s.encode())
 shutil.copytree(root,d/'before',dirs_exist_ok=True)
 native=[]; output=[]
 if expected:
  native.append(run(scope+['cargo','test','-j1','--','--test-threads','4'],root,d/'native-before.log').returncode)
  if 'src/main.rs' in files: output.append(run(scope+['cargo','run','-j1','--quiet'],root,d/'output-before.log').stdout.splitlines())
 p=run([binary,'do',str(root),'--explain',f'rename function {old} to {new}'],root,d/'apply.log')
 changed={f: (root/f).read_bytes()!=s.encode() for f,s in files.items()}
 edited=any(changed.values())
 exact=True
 # Expected identifier-only map; aliases differ in spelling, literals/comments must remain unchanged.
 import re
 for f,s in files.items():
  if not f.endswith('.rs'): want=s
  else:
   token=re.compile(r'//[^\r\n]*|"(?:\\.|[^"\\])*"|\b'+re.escape(old)+r'\b',re.DOTALL)
   want=token.sub(lambda m:new if m.group()==old else m.group(),s)
  if (root/f).read_bytes() != (want if expected and edited else s).encode(): exact=False
 if expected and edited:
  native.append(run(scope+['cargo','test','-j1','--','--test-threads','4'],root,d/'native-after.log').returncode)
  if 'src/main.rs' in files: output.append(run(scope+['cargo','run','-j1','--quiet'],root,d/'output-after.log').stdout.splitlines())
 shutil.copytree(root,d/'after',dirs_exist_ok=True)
 result={'case':name,'expected_edit':expected,'edited':edited,'exact':exact,'native':native,'output_equal':len(output)<2 or output[0]==output[1],'ok':edited==expected and exact and not any(native) and (len(output)<2 or output[0]==output[1])}
 results.append(result); print(json.dumps(result),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
