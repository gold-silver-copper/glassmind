import json
from pathlib import Path
out=Path(__file__).parent
fixtures=[]
def add(name,code,tests,old=None,new=None,kind='negative',extra=None):
 fixtures.append(dict(name=name,code=code,tests=tests,old=old,new=new,kind=kind,extra=extra or {}))
for t in ['u8','u16','u32','u64','u128','usize']:
 code=f'pub fn f(n: {t}, d: {t}) -> {t} {{ if d == 0 {{ return 0; }} n / d }}\n'
 tests=f'assert_eq!(f(0,3),0); assert_eq!(f(6,3),2); assert_eq!(f(7,3),3); assert_eq!(f({t}::MAX,2),{t}::MAX/2+1); assert_eq!(f(9,0),0);'
 add('width_'+t,code,tests,'n / d',f'::core::primitive::{t}::div_ceil(n, d)','positive')
add('typed_fields','pub struct Pair { pub count: u32, pub size: u32 }\npub fn f(p: &Pair) -> u32 { if p.size == 0 { return 0; } p.count / p.size }\n','for (n,d,w) in [(0,3,0),(6,3,2),(7,3,3),(u32::MAX,2,u32::MAX/2+1),(9,0,0)] { assert_eq!(f(&Pair{count:n,size:d}),w); }','p.count / p.size','::core::primitive::u32::div_ceil(p.count, p.size)','positive')
add('unicode','pub fn f(café: u16) -> u16 { café / 3 }\n','assert_eq!(f(0),0); assert_eq!(f(6),2); assert_eq!(f(7),3); assert_eq!(f(u16::MAX),21845);','café / 3','::core::primitive::u16::div_ceil(café, 3)','positive')
add('signed','pub fn f(n:i32)->i32 { n / 3 }\n','assert_eq!(f(7),3); assert_eq!(f(6),2);')
add('unknown','pub fn f()->u32 { let n = 7; n / 3 }\n','assert_eq!(f(),3);')
add('custom','#[derive(Clone,Copy)] pub struct N(pub u32); impl std::ops::Div for N { type Output = N; fn div(self,r:Self)->Self { N(self.0/r.0) } } impl N { pub fn div_ceil(self,_:Self)->Self { N(99) } } pub fn f(n:N,d:N)->N { n / d }\n','assert_eq!(f(N(7),N(3)).0,3); assert_eq!(f(N(6),N(3)).0,2);')
add('macro','pub fn f(n:u32)->u32 { std::convert::identity!() }\n'.replace('std::convert::identity!()', '{ macro_rules! q { () => { n / 3 } } q!() }'),'assert_eq!(f(7),3); assert_eq!(f(6),2);')
add('protected','pub fn f()->u32 { 1 }\n#[cfg(test)] fn helper(n:u32)->u32 { n / 3 }\n','assert_eq!(helper(7),3); assert_eq!(f(),1);')
add('rollback','pub fn f(n:u32)->u32 { n / 3 }\n','assert_eq!(f(7),3); assert_eq!(f(7),4);')
add('ambiguous','pub fn f(n:u32)->u32 { n / 2 + n / 3 }\n','assert_eq!(f(7),6);',kind='ambiguity')
add('legacy','mod early; mod late; pub fn f(n:u32)->u32 { n * 3 }\n','assert_eq!(f(6),2); assert_eq!(f(9),3);','*','/','legacy',{'src/early.rs':'pub fn q(n:u32)->u32 { n / 3 }\n','src/late.rs':'pub fn old(n:u32)->u32 { n + 1 }\n'})
(out/'fixtures.json').write_text(json.dumps(fixtures,indent=2,ensure_ascii=False)+'\n')
