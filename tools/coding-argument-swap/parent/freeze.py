from pathlib import Path
import hashlib,json,datetime
out=Path(__file__).parent;cases=[]
def helper(t):return f'pub fn distance(a:{t},b:{t})->{t} {{ if a >= b {{ a-b }} else {{ 0 }} }}\n'
def tests(t,expr,formatted=False):
 rows=[]
 for a,b in [(0,0),(1,0),(0,1),(11,4)]:
  expected=max(a-b,0);want=f'"value: {expected}"' if formatted else str(expected)
  rows.append(f'assert_eq!(distance({a},{b}),{expected}); assert_eq!({expr.format(a=a,b=b)},{want});')
 rows.append(f'assert_eq!(distance({t}::MAX,0),{t}::MAX);')
 return '\n#[cfg(test)] mod tests { use super::*; #[test] fn native(){'+''.join(rows)+'}}\n'
def add(name,kind,source,old=None,new=None,**extra):cases.append(dict(name=name,kind=kind,source=source,old=old,new=new,**extra))
for t in ['u8','u16','u32','u64','i32','i64']:
 src=helper(t)+f'pub fn value(left:{t},right:{t})->{t} {{ distance(right,left) }}\n'+tests(t,'value({a},{b})');add('direct_'+t,'repair',src,'distance(right,left)','distance(left,right)')
src=helper('u64')+'pub struct Pair {pub left:u64,pub right:u64}\npub fn value(p:&Pair)->String { format!("value: {}",distance(p.right,p.left)) }\n'+tests('u64','value(&Pair{{left:{a},right:{b}}})',True);add('format_fields','repair',src,'distance(p.right,p.left)','distance(p.left,p.right)')
src=helper('u32')+'pub fn value(π:u32,δ:u32)->u32 { /* café */ distance(δ,π) }\n'+tests('u32','value({a},{b})');add('unicode','repair',src,'distance(δ,π)','distance(π,δ)')
src=helper('u32')+'pub fn value(left:u32,right:u32)->String { ::std::format!("value: {}",distance(right,left)) }\n'+tests('u32','value({a},{b})',True);add('qualified_format','repair',src,'distance(right,left)','distance(left,right)')
src=helper('u32')+'pub fn value(left:u32,right:u32)->u32 { let target=distance; target(right,left) }\n'+tests('u32','value({a},{b})');add('callee_alias_unknown','unchanged',src)
src=helper('u32')+'macro_rules! format { ($s:literal,$v:expr)=>{::std::format!("value: {}",$v)} }\npub fn value(left:u32,right:u32)->String { format!("custom",distance(right,left)) }\n'+tests('u32','value({a},{b})',True);add('custom_format_macro','unchanged',src)
src=helper('u32')+'pub fn value(left:u32,right:u32)->(u32,u32) {let mut trace=0;let v=distance({trace=trace*10+1;right},{trace=trace*10+2;left});(v,trace)}\n#[cfg(test)]mod tests{use super::*;#[test]fn native(){assert_eq!(distance(11,4),7);assert_eq!(distance(4,11),0);assert_eq!(value(11,4),(7,12));}}\n';add('effectful_arguments','unchanged',src)
src=helper('u32')+'pub fn value(a:u32,b:u32)->u32 {distance(b,a)+distance(4,a)}\n#[cfg(test)]mod tests{use super::*;#[test]fn native(){assert_eq!(distance(5,4),1);assert_eq!(distance(4,5),0);assert_eq!(distance(7,2),5);assert_eq!(value(5,4),1);}}\n';add('competing_repairs','unresolved',src,alternatives=[['distance(b,a)','distance(a,b)'],['distance(4,a)','distance(a,4)']],distinguishing_input=[7,2],expected_outputs=[5,3])
(out/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
(out/'PLAN.md').write_text('''Parent argument-swap cases frozen before worker inspection:9native positives (six widths,signed/unsigned,Unicode,field operands insideformat,qualifiedstandardmacro),3unchangednegative controls(calleealias,customformatmacro,statefulargs),1unresolved competingrepair. Each original must compile and fail executed tests. Direct helper assertions prevent repairing helper arithmetic instead of the reversedcall. Repair exactlyone designatedcall; preserve every other sourcebyte including formatliteral andcomments. Harmless comma/operand whitespace changes inside designatedreplacementcall may be normalized; no other expected-result changes. All originalpackage/testbytes preserved, actual main do --explain command plus postnativeverification required. Unresolvedcase must stayunchanged andreport competing verifiedchoices orretain nativeambiguity evidence; merely unsupported/no passingcandidate is not sufficient. Independently replay thetwo supplied alternatives againstsamepublictests andtheir differing outputs at(7,2). Baseline expectations are hypotheses until measured; no gold/hiddenmaterial. MIT OR Apache-2.0 authored development fixtures, not benchmark claims.\n''')
files=['cases.json','PLAN.md','freeze.py'];(out/'freeze.json').write_text(json.dumps({'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'cases':len(cases),'sha256':{f:hashlib.sha256((out/f).read_bytes()).hexdigest() for f in files}},indent=2)+'\n');print((out/'freeze.json').read_text())
