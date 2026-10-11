import json
from pathlib import Path
cases=[]
def add(name,code,test,new=None,count=0):
 cases.append(dict(name=name,code=code,test=test,new=new,count=count))
def body(t='i32',op='>',arg='xs',acc='best',elem='x',field=False):
 prefix=f'pub struct Data {{ pub values: Vec<{t}> }}\n' if field else ''
 coll=f'{arg}.values' if field else arg
 ty='Data' if field else f'[{t}]'
 it=f'&{coll}' if field else coll
 return prefix+f'pub fn reduce({arg}: &{ty}) -> Option<{t}> {{\n if {coll}.is_empty() {{ return None; }}\n let mut {acc} = 0;\n for &{elem} in {it} {{ if {elem} {op} {acc} {{ {acc} = {elem}; }} }}\n Some({acc})\n}}\n'
for t in ['i8','i16','i32','i64','i128','isize']:
 for op,kind in [('>','max'),('<','min')]:
  field=(t in ['i32','i128'])
  code=body(t,op,'数据','结果','项',field)
  call='reduce(&Data { values: v.to_vec() })' if field else 'reduce(v)'
  test=f'''#[test] fn boundaries() {{
   fn check(v: &[{t}]) {{ assert_eq!({call},v.iter().copied().{kind}()); }}
   check(&[]); check(&[{t}::MIN]); check(&[{t}::MAX]); check(&[0]);
   check(&[-9,-2,-6]); check(&[2,9,6]); check(&[{t}::MIN,0,{t}::MAX]);
  }}'''
  add(f'{t}_{kind}',code,test,f'::core::primitive::{t}::{"MIN" if op==">" else "MAX"}',1)
base=body()
test='#[test] fn fails() { assert_eq!(reduce(&[-8,-2]),Some(-2)); }'
variants={
 'unguarded':base.replace('if xs.is_empty() { return None; }',''),
 'effectful':base.replace('best = x;', 'best = std::hint::black_box(x);'),
 'conditional':base.replace('best = x;', 'if x != -99 { best = x; }'),
 'shadowing':base.replace('if x > best', 'let best = 0; if x > best').replace('let best = 0;', 'let mut best = 0;'),
 'altered_result':base.replace('Some(best)','Some(best + 0)'),
 'extra_statement':base.replace('let mut best', 'let _ = xs.len(); let mut best'),
 'alias':'type Number = i32;\n'+base.replace('&[i32]','&[Number]'),
 'custom_deref':'pub struct Items(Vec<i32>); impl std::ops::Deref for Items { type Target = [i32]; fn deref(&self)->&[i32]{ &self.0 } }\n'+base.replace('&[i32]','&Items'),
 'attribute':base.replace('pub fn','#[inline]\npub fn'),
 'macro':base.replace('best = x;', 'best = { std::dbg!(x) };'),
 'iterator':base.replace('in xs','in xs.iter()'),
 'unknown_flow':base.replace('if x > best','if x == -99 { break; } if x > best'),
}
for n,c in variants.items():
 tt=test.replace('reduce(&[-8,-2])','reduce(&Items(vec![-8,-2]))') if n=='custom_deref' else test
 add(n,c,tt)
a=base.replace('reduce','left'); b=base.replace('reduce','right')
add('two_passing',a+b,'#[test] fn either() { assert!(left(&[-8,-2]) == Some(-2) || right(&[-8,-2]) == Some(-2)); }',count=2)
Path('tools/coding-extremum/contracts.json').write_text(json.dumps(cases,indent=2)+'\n')
