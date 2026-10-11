"""Authored public development contracts; frozen before production edits."""
import json
from pathlib import Path
cases=[]
def add(name,code,test,old=None,new=None):
 cases.append(dict(name=name,code=code,test=test,old=old,new=new))
for ty,lo,hi in [('u8','0','255'),('u16','0','65535'),('u32','0','4294967295'),('u64','0','18446744073709551615'),('u128','0','340282366920938463463374607431768211455'),('i64','-9223372036854775808','9223372036854775807')]:
 code=f'fn pair(a:{ty}, b:{ty})->({ty},{ty}) {{ (a,b) }}\npub fn run(a:{ty}, b:{ty})->({ty},{ty}) {{ pair(b, a) }}\n'
 add(ty,code,f'assert_eq!(probe::run({lo},{hi}),({lo},{hi}));', 'pair(b, a)','pair(a, b)')
add('unicode_trivia','fn pair(a:u16,b:u16)->(u16,u16){(a,b)}\npub fn run(café:u16,水:u16)->(u16,u16){ pair(\n 水 /* keep first slot */,\n café\n) }\n','assert_eq!(probe::run(0,65535),(0,65535));','pair(\n 水 /* keep first slot */,\n café\n)','pair(\n café /* keep first slot */,\n 水\n)')
add('fields','pub struct Packet { pub left:u32,pub right:u32 }\nfn pair(a:u32,b:u32)->(u32,u32){(a,b)}\npub fn run(p:&Packet)->(u32,u32){pair(p.right,p.left)}\n','assert_eq!(probe::run(&probe::Packet{left:0,right:4294967295}),(0,4294967295));','pair(p.right,p.left)','pair(p.left,p.right)')
add('format_fields','pub struct Packet { pub left:u64,pub right:u64 }\nfn first(a:u64,_b:u64)->u64{a}\npub fn run(p:&Packet)->String{format!("α{{}} {}!",first(p.right,p.left))}\n','assert_eq!(probe::run(&probe::Packet{left:0,right:18446744073709551615}),"α{} 0!");','first(p.right,p.left)','first(p.left,p.right)')
add('division_format','fn ratio(a:u64,b:u64)->u64{if b==0 {0} else {a*100/b}}\npub fn run(a:u64,b:u64)->String{format!("{}%",ratio(b,a))}\n','assert_eq!(probe::run(2,5),"40%"); assert_eq!(probe::run(1,0),"0%"); assert_eq!(probe::run(0,1),"0%");','ratio(b,a)','ratio(a,b)')
base='fn pair(a:u8,b:u8)->(u8,u8){(a,b)}\n'
add('wrong_rollback',base+'pub fn run(a:u8,b:u8)->(u8,u8){pair(b,a)}\n','assert_eq!(probe::run(1,2),(3,4));')
add('two_passing',base+'pub fn run(a:u8,b:u8,c:u8,d:u8)->((u8,u8),(u8,u8)){(pair(b,a),pair(d,c))}\n','let r=probe::run(1,2,3,4); assert!(r.0==(1,2) || r.1==(3,4));')
add('callee_shadow',base+'pub fn run(a:u8,b:u8)->(u8,u8){let pair=|a,b|(a,b);pair(b,a)}\n','assert_eq!(probe::run(1,2),(1,2));')
add('operand_shadow',base+'pub fn run(a:u8,b:u8)->(u8,u8){let a=b;pair(b,a)}\n','assert_eq!(probe::run(1,2),(1,2));')
add('stateful',base+'pub fn run(a:u8,b:u8)->(u8,u8){let mut x=b;pair({x=a;x},b)}\n','assert_eq!(probe::run(1,2),(2,1));')
add('opaque_macro',base+'macro_rules! pass {($e:expr)=>{$e}}\npub fn run(a:u8,b:u8)->(u8,u8){pass!(pair(b,a))}\n','assert_eq!(probe::run(1,2),(1,2));')
add('custom','pub struct Scalar(pub u8);\nfn pair(a:Scalar,b:Scalar)->(u8,u8){(a.0,b.0)}\npub fn run(a:u8,b:u8)->(u8,u8){pair(Scalar(b),Scalar(a))}\n','assert_eq!(probe::run(1,2),(1,2));')
add('format_override','macro_rules! format {($s:literal,$e:expr)=>{std::format!($s,$e)}}\nfn first(a:u8,_b:u8)->u8{a}\npub fn run(a:u8,b:u8)->String{format!("{}",first(b,a))}\n','assert_eq!(probe::run(1,2),"1");')
add('named_format','fn first(a:u8,_b:u8)->u8{a}\npub fn run(a:u8,b:u8)->String{format!("{x}",x=first(b,a))}\n','assert_eq!(probe::run(1,2),"1");')
add('imported','use std::cmp::min;\npub fn run(a:u8,b:u8)->u8{min(b,a)}\n','assert_eq!(probe::run(1,2),2);')
if __name__=='__main__':
 Path(__file__).with_name('contracts.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
