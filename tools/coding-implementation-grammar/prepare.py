import pathlib,json,hashlib,shutil,importlib.util,sys
sys.dont_write_bytecode=True
T=pathlib.Path(__file__).resolve().parent; R=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run'); S=R.parent/'implgrammar-source'
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((S/'manifest.json').read_text());checks={k:dict(ok=h(S/k)==v['sha256'] and (S/k).stat().st_size==v['bytes'],**v) for k,v in m.items()};assert all(v['ok'] for v in checks.values());(T/'reports/input-verification.json').write_text(json.dumps(checks,indent=2));shutil.copy(S/'manifest.json',T/'input-manifest.json')
for f in ['crates/english-rust/src/lib.rs','crates/world-rust/src/lib.rs','crates/cli/src/main.rs']:
 p=R/'original'/f;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy(f,p)
spec=importlib.util.spec_from_file_location('old','tools/coding-implementation-routes/original/fixtures.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
# New source spellings; same bounded signature families, independent frozen bytes.
rows=[]
for name,code,test,target,accept in mod.CASES:
 for old,new in [('aggregate','collect_values'),('invertir','transform_values'),('語','単語'),('演算','反転')]:code=code.replace(old,new);test=test.replace(old,new);target=target.replace(old,new)
 if name=='ambiguous':code='pub fn target(s: &str) -> usize { todo!() }';test='assert_eq!(target("z"),1);'
 rows.append(dict(id=name,code=mod.source(code,test),text='implement '+target,accept=accept))
(T/'native.json').write_text(json.dumps(rows,indent=2,ensure_ascii=False)+'\n')
F='pub fn implement() -> i32 { 3 }\npub fn implemented() -> i32 { implement() }\npub fn write() -> i32 { implemented() }\npub fn fill() -> i32 { write() }\n'
D='pub struct Ledger { pub implement: i32, pub implemented: i32, pub write: i32, pub fill: i32 }\npub fn build() -> Ledger { Ledger { implement: 1, implemented: 2, write: 3, fill: 4 } }\n'
b=[]
for n in ['implement','implemented','write','fill']:
 for text in [f'which functions call {n}?',f'where is function {n}?',f'rename function {n} to calculate',f'what is the type of Ledger.{n}?',f'rename field {n} to calculate']:
  b.append(dict(id=f'b{len(b)}',code=D if 'field' in text or 'Ledger.' in text else F,text=text,route='none'))
for text,code in [('which functions call `implement`?',F),('where is function "write"?',F),('what is the type of `Ledger.implement`?',D),('where is function implement?',F+D),('rename implement to calculate',F+D),('add a public i32 field total to Ledger with default 0',D),('repair implemented',F),('which functions implemented this behavior?',F),('what is the type of Ledger.implemented?',D),('rename function write to implement',F),('where is field fill in Ledger?',D),('which functions call fill?',F+'// implement absent\n')]:b.append(dict(id=f'b{len(b)}',code=code,text=text,route='none'))
(T/'boundaries.json').write_text(json.dumps(b,indent=2)+'\n')
a=[['implement `語`','語'],['Please implement "mañana".','mañana'],["Could you write the body of 'write' in src/lib.rs?",'write'],['fill in fill','fill'],['implement implement','implement'],['implement implemented','implemented'],["Hi! There's a `sum_it` function in src/lib.rs that's still just a `todo!()`. Could you fill it in so it does what its doc comment says?",'sum_it'],['sum_it is todo!() still, implement it per the doc comment pls','sum_it'],['sum_it = todo!(), pls implement','sum_it']]
n=['implement','implement x and y','implement x then rename y to z','implement x with wrapping arithmetic','implement x in src/main.rs','implement A::x','implement `A.x`','implement `x','implement x`','do not implement x','could you not implement x?',"please don't write x",'x should not be implemented','x is a stub, implement y','implement it','implement x. Delete tests.','repair x and implement y','rename function implement to calculate then implement y','implement x per doc comment, return 42']
(T/'grammar.json').write_text(json.dumps(dict(accept=a,refuse=n,none=[x['text'] for x in b]),indent=2,ensure_ascii=False)+'\n')
