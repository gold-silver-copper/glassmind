"""Public authored development contracts; no external corpus. MIT OR Apache-2.0."""
import json
from pathlib import Path
cases=[]
manifest='[package]\nname="roles"\nversion="0.0.0"\nedition="2021"\n[workspace]\n'
def add(label, request, old='caller', new=None, field=False, extra='', refuse=False):
 if field:
  source=f'pub struct Record {{ pub {old}: u32 }}\npub fn observe() -> u32 {{ Record {{ {old}: 7 }}.{old} }}\n'
  expected=source.replace(old,new) if new and not refuse else source
  located=f'`Record::{old}` is defined at src/lib.rs:1.\n'
 else:
  source=f'pub fn {old}() -> u32 {{ 7 }}\npub fn observe() -> u32 {{ {old}() }}\n'
  expected=source.replace(old,new) if new and not refuse else source
  located=f'`{old}` is defined at src/lib.rs:1.\n'
 source+=extra+f'// Preserve {old} in comments.\npub const NOTE: &str = "{old}";\n'
 expected+=extra+f'// Preserve {old} in comments.\npub const NOTE: &str = "{old}";\n'
 files={'Cargo.toml':manifest,'src/lib.rs':source,'tests/behavior.rs':'#[test]\nfn preserved() { assert_eq!(roles::observe(), 7); }\n'}
 cases.append(dict(id=label,request=request,files=files,expected={**files,'src/lib.rs':expected},reply=None if new or refuse else located,positive=not refuse,mutation=new is not None and not refuse))
for name in ['caller','renamed','called','drain_events','café']:
 for quote,tag in [('', 'plain'),('`','backtick'),('"','double')]:
  add(f'query-{name}-{tag}',f'where is function {quote}{name}{quote}?',name)
for name in ['called','renamed','drain_events']:
 add('field-query-'+name,f'where is field {name} of struct Record?',name,field=True)
 add('field-control-'+name,f'where is field `{name}` of struct `Record`?',name,field=True)
for old,new in [('caller','implement_again'),('implement','renamed'),('weigh','called'),('drain_events','read'),('café','naïve')]:
 add('rename-'+old,f'rename function {old} to {new}',old,new)
 add('rename-control-'+old,f'rename function `{old}` to `{new}`',old,new)
# Field world support is independently checked using the quoted controls.
for old,new in [('called','saved'),('renamed','read')]:
 add('field-rename-'+old,f'rename field {old} on Record to {new}',old,new,field=True)
 add('field-rename-control-'+old,f'rename field `{old}` on `Record` to `{new}`',old,new,field=True)
for tag,request,extra in [
 ('negation','do not rename function caller to renamed',''),
 ('multi','rename function caller to renamed and rename function observe to calculate',''),
 ('suffix','rename function caller to renamed inexplicable_suffix',''),
 ('collision','rename function caller to renamed','pub fn renamed() {}\n'),
 ('ambiguous','rename `caller` to renamed','pub struct Other { pub caller: u32 }\n'),
 ('two-functions','rename function caller to renamed','pub mod nested { pub fn caller() -> u32 { 3 } }\n'),
 ('wrong-kind','rename field caller to renamed',''),
]: add(tag,request,'caller','renamed',extra=extra,refuse=True)
Path('tools/coding-identifier-roles/contracts.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
