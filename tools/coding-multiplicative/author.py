"""Public authored source fixtures; no benchmark material or generated answers."""
import json
from pathlib import Path
out = Path(__file__).parent / 'fixtures.json'
cases=[]
def case(name,code,test,old=None,new=None,file='src/lib.rs',extra=None,kind='positive'):
    files={'Cargo.toml':'[package]\nname="probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n',file:code,'tests/visible.rs':test}
    if file!='src/lib.rs': files['src/lib.rs']='pub mod units;\n'
    files.update(extra or {})
    cases.append(dict(name=name,kind=kind,files=files,file=file,old=old,new=new))
for name,ty,op,inputs,outputs in [
 ('signed_div_to_mul','i32','/',[-100,-1,0,1,100],[-400,-4,0,4,400]),
 ('signed_mul_to_div','i32','*',[-100,-7,0,7,100],[-25,-1,0,1,25]),
 ('unsigned_div_to_mul','u16','/',[0,1,2,16383],[0,4,8,65532]),
 ('unsigned_mul_to_div','u16','*',[0,1,3,16383],[0,0,0,4095]),
 ('float_div_to_mul','f64','/',[-8.0,-0.5,0.0,0.25,8.0],[-32.0,-2.0,0.0,1.0,32.0]),
 ('float_mul_to_div','f64','*',[-8.0,-0.5,0.0,0.25,8.0],[-2.0,-0.125,0.0,0.0625,2.0]),
 ('module_unicode','i64','/',[-2,0,2,100],[-8,0,8,400]),
 ('module_signed_division','i64','*',[-99,-1,0,1,99],[-24,0,0,0,24]),
]:
    module=name.startswith('module'); arg='café' if 'unicode' in name else 'x'; factor='4.0' if ty=='f64' else '4'
    code=f'pub fn scale({arg}: {ty}) -> {ty} {{ {arg} {op} {factor} }}\n'
    code+='// * / are not proposals here.\nconst _TEXT: &str = "x * 4 / 4";\n'
    tests='use probe::'+('units::' if module else '')+'scale;\n#[test] fn boundaries() {\n'
    tests+=''.join(f'assert_eq!(scale({x}), {y});\n' for x,y in zip(inputs,outputs))+'}\n'
    case(name,code,tests,f'{arg} {op} {factor}',f'{arg} {"*" if op=="/" else "/"} {factor}',file='src/units.rs' if module else 'src/lib.rs')
case('macro_exclusion','pub fn scale(x:i32)->i32 { std::convert::identity({ macro_rules! calc { ($x:expr) => { $x * 4 }; } calc!(x) }) }\n', '#[test] fn fails(){ assert_eq!(probe::scale(8),2); assert_eq!(probe::scale(4),1); }\n',kind='negative')
case('assignment_exclusion','pub fn scale(mut x:i32)->i32 { x *= 4; x }\n','#[test] fn fails(){ assert_eq!(probe::scale(8),2); }\n',kind='negative')
case('unsafe_zero_divisor','pub fn scale(x:i32)->i32 { x * 0 }\n','#[test] fn fails(){ assert_eq!(probe::scale(0),0); assert_eq!(probe::scale(8),8); }\n',kind='negative')
case('two_edits_rollback','pub fn a(x:i32)->i32 { x * 4 }\npub fn b(x:i32)->i32 { x * 4 }\n','#[test] fn fails(){ assert_eq!(probe::a(8),2); assert_eq!(probe::b(8),2); assert_eq!(probe::a(0),0); }\n',kind='negative')
case('compile_failed_candidate','pub struct Only;\nimpl std::ops::Mul<i32> for Only { type Output=i32; fn mul(self,r:i32)->i32 {r} }\npub fn scale()->i32 { Only * 4 }\n','#[test] fn fails(){ assert_eq!(probe::scale(),8); }\n',kind='negative')
case('test_exclusion','pub fn scale(x:i32)->i32 { x }\n#[cfg(test)] mod helper;\n#[cfg(test)] mod inline { #[test] fn fails(){ assert_eq!(8*4,2); } }\n','#[test] fn fails(){ assert_eq!(8*4,2); }\n',extra={'src/helper.rs':'#[test] fn fails(){ assert_eq!(8*4,2); }\n'},kind='negative')
case('old_cross_file_priority','pub mod early;\npub mod late;\n','#[test] fn works(){ assert_eq!(probe::early::f(4)+probe::late::g(4),9); }\n',extra={'src/early.rs':'pub fn f(x:i32)->i32 { x / 2 }\n','src/late.rs':'pub fn g(x:i32)->i32 { x - 3 }\n'},kind='legacy')
cases[-1].update(file='src/late.rs',old='x - 3',new='x + 3')
case('native_ambiguity','pub fn scale(x:i32)->i32 { x / 2 - 3 }\n','#[test] fn underconstrained(){ assert_eq!(probe::scale(4),5); }\n',kind='ambiguity')
out.write_text(json.dumps(cases,indent=2)+'\n')
