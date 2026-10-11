"""Parent-authored extremum contracts, independent of candidate code."""
from pathlib import Path
import hashlib,json
base=Path(__file__).parent;cases=[]
def fold(ty,mode,name='reduce',var='best',field=False):
 op='>' if mode=='max' else '<';arg='batch: &Batch' if field else f'values: &[{ty}]';values='batch.values' if field else 'values';iterator='&batch.values' if field else 'values'
 prefix=f'pub struct Batch {{ pub values: Vec<{ty}> }}\n' if field else ''
 return prefix+f'pub fn {name}({arg}) -> Option<{ty}> {{\n    if {values}.is_empty() {{ return None; }}\n    let mut {var} = 0;\n    for &element in {iterator} {{\n        if element {op} {var} {{ {var} = element; }}\n    }}\n    Some({var})\n}}\n'
def tests(ty,mode,field=False):
 call=lambda v:f'reduce(&Batch {{ values: vec!{v} }})' if field else f'reduce(&{v})'
 data=[('[]','None'),('[0]','Some(0)'),(f'[{ty}::MIN]',f'Some({ty}::MIN)'),(f'[{ty}::MAX]',f'Some({ty}::MAX)'),('[-8,-3,-6]','Some(-3)' if mode=='max' else 'Some(-8)'),('[8,3,6]','Some(8)' if mode=='max' else 'Some(3)'),('[-8,0,6]','Some(6)' if mode=='max' else 'Some(-8)'),('[6,0,-8]','Some(6)' if mode=='max' else 'Some(-8)')]
 return '#[cfg(test)] mod tests { use super::*; #[test] fn contract() {\n'+''.join(f'assert_eq!({call(v)}, {expected});\n' for v,expected in data)+'}}\n'
def positive(name,ty,mode,field=False,var='best'):
 source=fold(ty,mode,var=var,field=field)+tests(ty,mode,field);old=f'let mut {var} = 0;';bound='MIN' if mode=='max' else 'MAX'
 new=[f'let mut {var} = {prefix}{ty}::{bound};' for prefix in ['::core::primitive::','::std::primitive::','']]
 assert source.count(old)==1;cases.append(dict(name=name,kind='repair',source=source,old=old,new=new))
for ty in ['i8','i16','i32','i64','i128','isize']:
 for mode in ['max','min']:positive(ty+'_'+mode,ty,mode)
positive('field_max','i32','max',field=True);positive('field_min','i64','min',field=True);positive('unicode','i32','max',var='άριστο')
source=fold('i32','max').replace('    if values.is_empty() { return None; }\n','')+tests('i32','max').replace('reduce(&[]), None','reduce(&[]), Some(0)')
cases.append(dict(name='unguarded_empty_default',kind='unchanged',source=source))
source='type Reading = i32;\n'+fold('Reading','max')+tests('i32','max');cases.append(dict(name='unknown_type_alias',kind='unchanged',source=source))
source=fold('i32','max').replace('let mut best = 0;','let mut best = 0;\n    let mut updates = 0;').replace('best = element;','best = element; updates += 1;').replace('    Some(best)','    let _ = updates;\n    Some(best)')+tests('i32','max');cases.append(dict(name='effectful_update',kind='unchanged',source=source))
source=fold('i32','max',name='upper',var='first')+fold('i32','max',name='other',var='second')+'pub fn joined(a: &[i32], b: &[i32]) -> Option<i32> { Some(upper(a)? + other(b)?) }\n#[cfg(test)] mod tests { use super::*; #[test] fn contract(){ assert_eq!(upper(&[2,1]),Some(2)); assert_eq!(other(&[2,1]),Some(2)); assert_eq!(joined(&[-3],&[-3]),Some(-3)); } }\n'
alternatives=[dict(old=f'let mut {var} = 0;',new=f'let mut {var} = ::core::primitive::i32::MIN;',expected_probe=value) for var,value in [('first',-5),('second',-2)]]
cases.append(dict(name='competing_repairs',kind='unresolved',source=source,alternatives=alternatives,probe='joined(&[-5], &[-2])'))
path=base/'cases.json';assert not path.exists();path.write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n');(base/'freeze.json').write_text(json.dumps(dict(sha256=hashlib.sha256(path.read_bytes()).hexdigest(),total=len(cases),repairs=15,negative=3,competing=1),indent=2)+'\n');print((base/'freeze.json').read_text())
