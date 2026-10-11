"""Public authored contracts; frozen before implementation. No external assets."""
import json
from pathlib import Path
cases=[]
def add(name,code,test,count,accept,**kw):
 cases.append(dict(name=name,code=code,test=test,count=count,accept=accept,**kw))
struct='#[derive(Debug, Clone, PartialEq)] pub struct Row {pub x: i32, pub y: String}\n'
body='pub fn order(input: &[Row])->Vec<Row>{let mut values=input.to_vec(); values.sort_by(|a,b| a.x.cmp(&b.x).then_with(|| a.y.cmp(&b.y))); values}'
test='''fn r(x:i32,y:&str)->Row{Row{x,y:y.into()}} #[test] fn t(){
assert_eq!(order(&[r(-5,"b"),r(2,"z"),r(-5,"a"),r(2,"z")]),vec![r(2,"z"),r(2,"z"),r(-5,"a"),r(-5,"b")]);
assert_eq!(order(&[]),vec![]); assert_eq!(order(&[r(-9,"é")]),vec![r(-9,"é")]);}'''
add('signed_ties',struct+body,test,2,True)
add('unicode',struct+body.replace('values','résultat').replace('a.x.cmp(&b.x)','a.x /* π preserve */ .cmp(&b.x)'),test,2,True)
add('secondary',struct+body.replace('a.x.cmp(&b.x)','b.x.cmp(&a.x)').replace('a.y.cmp(&b.y)','b.y.cmp(&a.y)'),test,2,True)
add('correct',struct+body.replace('a.x.cmp(&b.x)','b.x.cmp(&a.x)'),test,2,False,original_pass=True)
for typ,lo,hi in [('i8','-3','5'),('i16','-3','5'),('i64','-3','5'),('i128','-3','5'),('isize','-3','5'),('u8','3','5'),('u16','3','5'),('u32','3','5'),('u64','3','5'),('u128','3','5'),('usize','3','5'),('bool','false','true'),('char',"'a'","'é'"),('String','"a".into()','"é".into()')]:
 s=f'#[derive(Debug,Clone,PartialEq)] pub struct Row{{pub x:{typ}}}'
 b='pub fn order(mut values:Vec<Row>)->Vec<Row>{values.sort_by(|a,b| a.x.cmp(&b.x));values}'
 t=f'#[test] fn t(){{assert_eq!(order(vec![Row{{x:{lo}}},Row{{x:{hi}}}]),vec![Row{{x:{hi}}},Row{{x:{lo}}}]);}}'
 add('type_'+typ,s+b,t,1,True)
add('typed_local',struct+body.replace('let mut values=input.to_vec()','let mut values:Vec<Row>=input.to_vec()'),test,2,True)
add('inline_tests',struct+body+'\n#[cfg(test)] mod tests {use super::*;'+test+'}',test,2,True)
# Refusal fixtures retain actual failing public behavior; no expected patches.
for name,code in [
 ('alias',struct+'type Count=i32;\n'+body.replace('let mut values','let mut values') .replace('input: &[Row]','input: &[Row]')),
 ('import','use std::cmp::Ord;\n'+struct+body),
 ('shadow',struct+body.replace('let mut values=input.to_vec();','let mut values=input.to_vec(); {let values=1; let _=values;}')),
 ('nested_param',struct+body.replace('let mut values=input.to_vec();','let mut values=input.to_vec(); {let a=1; let _=a;}')),
 ('effectful_receiver',struct+body.replace('values.sort_by','{ &mut values }.sort_by')),
 ('opaque_attribute',struct+body.replace('pub fn','#[inline] pub fn')),
 ('generic',struct+body.replace('fn order(', 'fn order<T>(')),
 ('macro',struct+body.replace('let mut values','let _=stringify!(x); let mut values')),
 ('custom_cmp',struct+'impl Row {fn cmp(&self,_:&Self)->std::cmp::Ordering{std::cmp::Ordering::Equal}}'+body),
 ('qualified',struct.replace('i32','std::primitive::i32')+body),
]:
 if name=='generic': continue # native call requires type; covered by structural guard test
 add(name,code,test,0,False)
# Two distinct repairs satisfy the same weak test but have different native outputs.
b2=body.replace('fn order','fn other')
t2='fn r(x:i32)->Row{Row{x,y:String::new()}} #[test] fn t(){assert!(order(&[r(1),r(2)])[0].x==2 || other(&[r(1),r(2)])[0].x==2);}'
add('competing',struct+body+'\n'+b2,t2,4,False,outputs=True)
Path('tools/coding-comparator/cases.json').write_text(json.dumps(cases,indent=2)+'\n')
