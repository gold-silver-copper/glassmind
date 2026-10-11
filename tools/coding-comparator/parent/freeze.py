from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent
cases=[]
def source(ty,values,assertion,chain=True):
    tail='.then_with(|| a.label.cmp(&b.label))' if chain else ''
    return f'''#[derive(Clone, Debug, PartialEq)]
pub struct Card {{ pub rank: {ty}, pub label: String }}
pub fn arrange(cards: &[Card]) -> Vec<Card> {{
    let mut ordered = cards.to_vec();
    ordered.sort_by(|a, b| a.rank.cmp(&b.rank){tail});
    ordered
}}
#[cfg(test)] mod tests {{ use super::*;
#[test] fn visible() {{ let xs = vec![{values}]; let ys = arrange(&xs); {assertion} }}
}}
'''
def card(rank,label):return f'Card {{ rank: {rank}, label: "{label}".into() }}'
values=', '.join([card('4','z'),card('9','z'),card('9','a'),card('4','a')])
base=source('u32',values,'assert_eq!(ys.iter().map(|x| (x.rank,x.label.as_str())).collect::<Vec<_>>(), vec![(9,"a"),(9,"z"),(4,"a"),(4,"z")]);')
old='a.rank.cmp(&b.rank)';new=['b.rank.cmp(&a.rank)','a.rank.cmp(&b.rank).reverse()']
cases.append(dict(name='unsigned_primary_tie',kind='repair',files={'src/lib.rs':base},file='src/lib.rs',old=old,new=new))
specs=[
 ('signed_primary','i64',', '.join([card('-9','x'),card('2','x'),card('-1','x')]),'assert_eq!(ys.iter().map(|x| x.rank).collect::<Vec<_>>(), vec![2,-1,-9]);'),
 ('string_primary','String',', '.join([card('"apple".into()','z'),card('"pear".into()','z'),card('"pear".into()','a')]),'assert_eq!(ys.iter().map(|x| (x.rank.as_str(),x.label.as_str())).collect::<Vec<_>>(), vec![("pear","a"),("pear","z"),("apple","z")]);'),
 ('char_unicode','char',', '.join([card("'a'",'x'),card("'é'",'x'),card("'z'",'x')]),"assert_eq!(ys.iter().map(|x| x.rank).collect::<Vec<_>>(), vec!['é','z','a']);"),
 ('bool_primary','bool',', '.join([card('false','x'),card('true','z'),card('true','a')]),'assert_eq!(ys.iter().map(|x| (x.rank,x.label.as_str())).collect::<Vec<_>>(), vec![(true,"a"),(true,"z"),(false,"x")]);'),
]
for name,ty,vals,test in specs:
    code=source(ty,vals,test)
    if name=='char_unicode':code=code.replace('ordered','ordonné').replace(old,'a.rank /* retained */ .cmp(&b.rank)')
    oldtext='a.rank /* retained */ .cmp(&b.rank)' if name=='char_unicode' else old
    news=['b.rank /* retained */ .cmp(&a.rank)','a.rank /* retained */ .cmp(&b.rank).reverse()'] if name=='char_unicode' else new
    cases.append(dict(name=name,kind='repair',files={'src/lib.rs':code},file='src/lib.rs',old=oldtext,new=news))
for name,code in [
 ('type_alias','type Rank = u32;\n'+base.replace('rank: u32','rank: Rank')),
 ('import_alias','use std::vec::Vec as Buffer;\n'+base.replace('-> Vec<Card>','-> Buffer<Card>')),
 ('custom_ord','#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct Key(u32);\n'+base.replace('rank: u32','rank: Key').replace('rank: 4','rank: Key(4)').replace('rank: 9','rank: Key(9)').replace('(x.rank,x.label.as_str())','(x.rank.0,x.label.as_str())')),
 ('computed_receiver',base.replace(old,'rank_of(a).cmp(&rank_of(b))')+'fn rank_of(x: &Card) -> u32 { x.rank }\n'),
]:cases.append(dict(name=name,kind='unchanged',files={'src/lib.rs':code}))

amb=source('u32',', '.join([card('1','a'),card('1','b'),card('2','a'),card('2','b')]),'assert!(ys[0].rank == 2 || ys[0].label == "b");')
cases.append(dict(name='competing_comparisons',kind='unresolved',files={'src/lib.rs':amb},alternatives=[{'src/lib.rs':amb.replace(old,new[0])},{'src/lib.rs':amb.replace('a.label.cmp(&b.label)','b.label.cmp(&a.label)')}],outputs=['[(2, "a"), (2, "b"), (1, "a"), (1, "b")]\n','[(1, "b"), (1, "a"), (2, "b"), (2, "a")]\n'],output_example='fn main(){let xs=vec!['+', '.join(card(str(n),label) for n,label in [(1,'a'),(1,'b'),(2,'a'),(2,'b')])+'];println!("{:?}",arrange(&xs).iter().map(|x|(x.rank,x.label.clone())).collect::<Vec<_>>());}\n'))
lib='''pub fn left(mut xs: Vec<u32>) -> Vec<u32> { xs.dedup(); xs.sort(); xs }
pub fn right(mut xs: Vec<u32>) -> Vec<u32> { xs.dedup(); xs.sort(); xs }
'''
main=source('u32',', '.join([card('1','a'),card('2','b')]),'assert!(probe::left(vec![7,8,7]).len()==2 || probe::right(vec![7,8,7]).len()==2 || ys[0].rank==2);',chain=False)+'fn main() {}\n'
pair='xs.dedup(); xs.sort();';replacement='xs.sort(); xs.dedup();'
i=lib.index(pair);j=lib.index(pair,i+1)
variants=[{'src/lib.rs':lib[:i]+replacement+lib[i+len(pair):],'src/main.rs':main},{'src/lib.rs':lib[:j]+replacement+lib[j+len(pair):],'src/main.rs':main},{'src/lib.rs':lib,'src/main.rs':main.replace(old,new[0])}]
cases.append(dict(name='old_dedup_veto',kind='unresolved',files={'src/lib.rs':lib,'src/main.rs':main},alternatives=variants,outputs=['([7, 8], [7, 7, 8], 1)\n','([7, 7, 8], [7, 8], 1)\n','([7, 7, 8], [7, 7, 8], 2)\n'],output_main='fn main(){let xs=vec!['+card('1','a')+','+card('2','b')+'];println!("{:?}",(probe::left(vec![7,8,7]),probe::right(vec![7,8,7]),arrange(&xs)[0].rank));}\n'))
(HERE/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
(HERE/'PLAN.md').write_text('''# Frozen comparator parent gate

Five typed field-order repairs (u32/i64/String/char/bool), four unchanged controls
(alias, import, custom Ord, computed receiver), and two ambiguity refusals. All
originals must compile and fail executed tests. Preserve every byte outside the
single comparison; allow operand exchange or the equivalent native reverse call.
Primary direction must preserve secondary tie order. Unicode/comment spans remain
exact. Both comparison alternatives actually pass the same public test and yield
different complete sequences. The two-file old-dedup case keeps lib.rs within its
older closed namespace while main.rs contains the typed record comparator. Both
old alternatives and the new alternative pass but yield three different outputs;
complete old-family ambiguity must prevent the new fallback.

Freeze before candidate inspection. The unchanged full765 runner must preserve
all212 previous passes and have0failures. PublicTRAIN selection remains fixed;
no hidden evaluation items, labels, generators or source contents are inspected.
''')
(HERE/'freeze.json').write_text(json.dumps({n:hashlib.sha256((HERE/n).read_bytes()).hexdigest() for n in ['cases.json','PLAN.md','freeze.py']},indent=2)+'\n')
print(len(cases),hashlib.sha256((HERE/'cases.json').read_bytes()).hexdigest())
