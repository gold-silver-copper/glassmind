# SPDX-License-Identifier: MIT OR Apache-2.0
"""Frozen, locally authored development sources and behavioral contracts."""
def source(ty='i32', base=1, lower=None, upper=None, binding=None, name='pick', rank='rank', seq='items'):
    lower = lower or (f'{rank} < 1' if base else f'{rank} < 0')
    upper = upper or f'{rank} as usize {">" if base else ">="} {seq}.len()'
    binding = binding if binding is not None else (f'let index = {rank} - 1;' if base else f'let index = {rank} + 0;')
    return f'pub fn {name}({seq}: &[i32], {rank}: {ty}) -> Option<i32> {{\n if {lower} || {upper} {{ return None; }}\n {binding}\n Some({seq}[index])\n}}\n'
POSITIVES = [
 ('one_i8',source('i8'), 'i8',1,'pick'),
 ('one_i16',source('i16',lower='1 > rank'), 'i16',1,'pick'),
 ('one_i32',source('i32',upper='items.len() < rank as usize'), 'i32',1,'pick'),
 ('one_i64',source('i64').replace('rank < 1 || rank as usize > items.len()', 'rank <= 0 || rank as usize > items.len()'), 'i64',1,'pick'),
 ('zero_i16',source('i16',0), 'i16',0,'pick'),
 ('zero_i32',source('i32',0,binding='').replace('items[index]','items[rank]'), 'i32',0,'pick'),
 ('split_named',source('i32',name='fetch',rank='position',seq='values').replace('position < 1 || position as usize > values.len()', 'position < 1').replace(' let index', ' if position as usize > values.len() { return None; }\n let index'), 'i32',1,'fetch'),
]
GOOD=source()
NEGATIVES = [
 ('absent',GOOD.replace(' if rank < 1 || rank as usize > items.len() { return None; }\n','')),
 ('no_lower',GOOD.replace('rank < 1 || ','')),
 ('no_upper',GOOD.replace(' || rank as usize > items.len()','')),
 ('zero_wrong_upper',source(base=0).replace('>=','>')),
 ('and',GOOD.replace(' || ',' && ')),
 ('no_return',GOOD.replace('return None;','None;')),
 ('mutation',GOOD.replace('rank: i32','mut rank: i32').replace(' let index',' rank = 0;\n let index')),
 ('shadow',GOOD.replace(' let index',' let rank = -1;\n let index')),
 ('overflow',GOOD.replace('rank - 1','rank + 1')),
 ('negative',GOOD.replace('rank < 1','rank < -5')),
 ('unsigned',GOOD.replace('rank: i32','rank: u32')),
 ('generic',GOOD.replace('fn pick(', 'fn pick<T>(')),
 ('custom',GOOD.replace('&[i32]','&Custom')),
 ('macro',GOOD.replace(' let index',' dbg!(rank);\n let index')),
 ('unknown_attr','#[unknown]\n'+GOOD),
 ('conditional','#[cfg(unix)]\n'+GOOD),
 ('join',GOOD.replace(' if rank < 1',' if true { if rank < 1').replace(' let index',' }\n let index')),
 ('two_indices',GOOD.replace('Some(items[index])','Some(items[index] + items[index])')),
 ('call',GOOD.replace('rank - 1','other(rank)')),
 ('alias','type Int = i32;\n'+GOOD.replace('rank: i32','rank: Int')),
 ('local_attr',GOOD.replace('let index','#[allow(unused)] let index')),
 ('upper_first',GOOD.replace('rank < 1 || rank as usize > items.len()', 'rank as usize > items.len() || rank < 1')),
]
# Upper-first is deliberately outside the supported short-circuit proof language.
def behavior(ty,base,name):
    return f'''#[test]
fn independent_contract() {{
 let data = [12, 34, 56];
 for n in [{ty}::MIN, -2, -1, 0, 1, 2, 3, 4, {ty}::MAX] {{
   let expected = match n {{ {base} => Some(12), {base+1} => Some(34), {base+2} => Some(56), _ => None }};
   assert_eq!(task::{name}(&data, n), expected, "rank {{n}}");
   assert_eq!(task::{name}(&[], n), None);
 }}
}}
'''
