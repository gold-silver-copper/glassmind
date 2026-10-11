"""Public authored development contracts; no evaluation data is used."""
import json
from pathlib import Path
cases=[]
def add(name,code,test,count=1):
 cases.append(dict(name=name,code=code,test=test,count=count,accept=count==1))
def func(ty='i32',name='xs',head='',middle='',first='dedup()',second='sort()'):
 return f'{head}pub fn f(mut {name}: Vec<{ty}>) -> Vec<{ty}> {{ {middle}{name}.{first}; /* preserved gap */ {name}.{second}; {name} }}'
numtest='''#[test] fn checks() { for (a,b) in [(vec![],vec![]),(vec![3],vec![3]),(vec![1,2,3],vec![1,2,3]),(vec![3,2,1],vec![1,2,3]),(vec![1,1,2,2],vec![1,2]),(vec![3,1,3,2,1],vec![1,2,3])] { assert_eq!(f(a),b); } }'''
for ty in ['i8','i32','i128','u16','u64','usize']:
 add('owned_'+ty,func(ty),numtest)
add('unicode',func(name='éléments'),numtest)
add('typed_local','pub fn f(input: Vec<i32>) -> Vec<i32> { let mut xs: Vec<i32> = input.into_iter().collect(); xs.dedup(); /* preserved gap */ xs.sort(); xs }',numtest)
stringcode='pub fn f(raw: &[&str]) -> Vec<String> { let mut xs: Vec<String> = raw.iter().map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect(); xs.dedup(); /* preserved gap */ xs.sort(); xs }'
stringtest='#[test] fn checks() { for (a,b) in [(vec![],vec![]),(vec!["Z"],vec!["z"]),(vec!["a","a","b"],vec!["a","b"]),(vec![" Z ","a","z","","A"],vec!["a","z"])] { assert_eq!(f(&a), b); } }'
add('string_pipeline',stringcode,stringtest)
add('inline_tests',func()+'\n#[cfg(test)] mod tests { #[test] fn check() { assert_eq!(super::f(vec![2,1,2]),vec![1,2]); } }',numtest)
add('alias_vec',func(head='type Vec<T> = std::vec::Vec<T>; '),numtest,0)
add('alias_scalar',func(head='type i32 = i64; '),numtest,0)
add('alias_string',stringcode.replace('Vec<String>','Vec<Text>')+' type Text = String;',stringtest,0)
add('import',func(head='use std::vec::Vec; '),numtest,0)
add('custom','pub struct Vec<T>(std::vec::Vec<T>); impl<T: Ord> Vec<T> { fn dedup(&mut self) { self.0.dedup(); } fn sort(&mut self) { self.0.sort(); } } pub fn f(mut xs: Vec<i32>) -> Vec<i32> { xs.dedup(); xs.sort(); xs } pub fn expose() -> std::vec::Vec<i32> { f(Vec(std::vec![2,1,2])).0 }','#[test] fn checks() { assert_eq!(expose(),std::vec![1,2]); }',0)
add('shadow',func(middle='let mut xs: Vec<i32> = xs; '),numtest,0)
add('wrong_receiver','pub fn f(mut xs: Vec<i32>) -> Vec<i32> { let mut ys: Vec<i32> = xs.clone(); xs.dedup(); ys.sort(); xs }',numtest,0)
add('effect',func().replace('/* preserved gap */','xs.reverse();'),numtest,0)
add('dedup_by',func(first='dedup_by(|a,b| a == b)'),numtest,0)
add('sort_by',func(second='sort_by(|a,b| a.cmp(b))'),numtest,0)
add('unresolved_local','pub fn f(input: Vec<i32>) -> Vec<i32> { let mut xs = input; xs.dedup(); xs.sort(); xs }',numtest,0)
add('attribute',func(head='#[inline] '),numtest,0)
add('macro',func(middle='let _other = vec![1]; '),numtest,0)
add('generic','pub fn f<T: Ord>(mut xs: Vec<T>) -> Vec<T> { xs.dedup(); xs.sort(); xs }',numtest,0)
# Original fails without scalar/loop alternative. Either one-pair swap passes OR.
comp=func().replace('fn f(', 'fn left(')+'\n'+func().replace('fn f(', 'fn right(')
test='#[test] fn checks() { assert!(left(vec![2,1,2]) == vec![1,2] || right(vec![2,1,2]) == vec![1,2]); }'
add('two_passing',comp,test,2)
Path(__file__).with_name('cases.json').write_text(json.dumps(cases,indent=2)+'\n')
