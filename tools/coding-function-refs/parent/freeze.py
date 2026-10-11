from pathlib import Path
import hashlib,json
p=Path(__file__).parent
cases=[]
manifest='[package]\nname="identity_parent"\nversion="0.1.0"\nedition="2024"\n'
func='pub fn parse_num(x: i32) -> i32 { x * 3 + 1 }\n'
test='#[test] fn behavior() { assert_eq!(run(), 43); }\n'
def add(name,files,expected,required=False):
 files={'Cargo.toml':manifest,**files}
 cases.append(dict(name=name,files=files,expected={'Cargo.toml':manifest,**expected} if expected is not None else None,required=required,request='rename function parse_num to decode_num'))
def pos(name,files,edits,required=True):
 expected=dict(files)
 for path,pairs in edits.items():
  for old,new in pairs: expected[path]=expected[path].replace(old,new)
 add(name,files,expected,required)
# Expected edits are frozen explicitly, independently of candidate source.
pos('grouped_reexport_callback',{'src/lib.rs':'mod math;\npub use math::{parse_num, Marker};\npub fn run() -> i32 { [14].into_iter().map(parse_num).sum() }\n'+test,'src/math.rs':func+'pub struct Marker;\n'}, {'src/lib.rs':[('parse_num','decode_num')],'src/math.rs':[('fn parse_num','fn decode_num')]})
pos('aliased_import',{'src/lib.rs':'mod math;\nuse math::parse_num as chosen;\npub fn run() -> i32 { chosen(14) }\n'+test,'src/math.rs':func}, {'src/lib.rs':[('math::parse_num','math::decode_num')],'src/math.rs':[('fn parse_num','fn decode_num')]})
pos('aliased_reexport',{'src/lib.rs':'mod math;\npub use math::parse_num as chosen;\npub fn run() -> i32 { chosen(14) }\n'+test,'src/math.rs':func}, {'src/lib.rs':[('math::parse_num','math::decode_num')],'src/math.rs':[('fn parse_num','fn decode_num')]})
pos('qualified_nested',{'src/lib.rs':'pub mod math;\npub fn run() -> i32 { self::math::parse_num(14) }\n'+test,'src/math.rs':func}, {'src/lib.rs':[('math::parse_num','math::decode_num')],'src/math.rs':[('fn parse_num','fn decode_num')]})
pos('function_pointer',{'src/lib.rs':func+'pub fn run() -> i32 { let callback: fn(i32) -> i32 = parse_num; callback(14) }\n'+test}, {'src/lib.rs':[('parse_num','decode_num')]})
pos('lib_bin_identity',{'src/lib.rs':func+'pub fn run() -> i32 { parse_num(14) }\n'+test,'src/main.rs':'use identity_parent::parse_num as chosen;\nfn main() { println!("{}", chosen(14)); }\n'}, {'src/lib.rs':[('parse_num','decode_num')],'src/main.rs':[('::parse_num','::decode_num')]})
pos('test_glob',{'src/lib.rs':func+'pub fn run() -> i32 { parse_num(14) }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(parse_num(14), 43); } }\n'}, {'src/lib.rs':[('parse_num','decode_num')]})
pos('strings_comments_crlf',{'src/lib.rs':('// parse_num remains a comment; café\n'+func+'pub const NOTE: &str = "parse_num";\npub fn run() -> i32 { parse_num(14) }\n'+test).replace('\n','\r\n')}, {'src/lib.rs':[('fn parse_num','fn decode_num'),('{ parse_num(14)','{ decode_num(14)')]},False)
# Conservative refusals are required for unknown identity-changing contexts.
add('destination_collision',{'src/lib.rs':func+'pub fn decode_num(x:i32)->i32{x+8}\npub fn run()->i32{parse_num(14)}\n'+test},None)
add('custom_macro',{'src/lib.rs':func+'macro_rules! invoke { ($f:ident) => { $f(14) }; }\npub fn run()->i32{invoke!(parse_num)}\n'+test},None)
add('shadowed_assert_macro',{'src/lib.rs':func+'macro_rules! assert_eq { ($f:ident) => { $f(14) }; }\npub fn run()->i32{assert_eq!(parse_num)}\n#[test] fn behavior(){assert!(run()==43);}\n'},None)
# These are legal binding distinctions. Either safe refusal or exactly specified edits is acceptable.
pos('parameter_homonym',{'src/lib.rs':func+'fn local(parse_num:i32)->i32{parse_num}\npub fn run()->i32{parse_num(local(14))}\n'+test},{'src/lib.rs':[('fn parse_num(','fn decode_num('),('{parse_num(local','{decode_num(local')]},False)
pos('closure_homonym',{'src/lib.rs':func+'pub fn run()->i32{let add=|parse_num:i32|parse_num+1; parse_num(add(13))}\n'+test},{'src/lib.rs':[('fn parse_num(','fn decode_num('),('; parse_num(add','; decode_num(add')]},False)
pos('field_homonym',{'src/lib.rs':func+'struct Holder{parse_num:i32}\npub fn run()->i32{let h=Holder{parse_num:14}; parse_num(h.parse_num)}\n'+test},{'src/lib.rs':[('fn parse_num(','fn decode_num('),('; parse_num(h','; decode_num(h')]},False)
add('duplicate_functions',{'src/lib.rs':'mod left { pub fn parse_num(x:i32)->i32{x*3+1} }\nmod right { pub fn parse_num(x:i32)->i32{x+1} }\npub fn run()->i32{left::parse_num(14)+right::parse_num(0)-1}\n'+test},None)
pos('unicode_identifier_context',{'src/lib.rs':func+'pub fn run()->i32{let café=14; parse_num(café)}\n'+test},{'src/lib.rs':[('parse_num','decode_num')]},False)
data=(json.dumps(cases,ensure_ascii=False,indent=2)+'\n').encode()
assert not (p/'cases.json').exists()
(p/'cases.json').write_bytes(data)
(p/'freeze.json').write_text(json.dumps({'cases_sha256':hashlib.sha256(data).hexdigest(),'cases':len(cases),'required_positive':sum(c['required'] for c in cases),'created_before_candidate_inspection':True,'criterion':'Every edit must byte-match frozen identity mapping and pass unchanged before/after native behavior; conservative refusals permitted except required positives. No scoring from candidate implementation.'},indent=2)+'\n')
print((p/'freeze.json').read_text())
