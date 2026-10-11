#!/usr/bin/env python3
"""Authored visible contracts, MIT OR Apache-2.0. No downloaded data."""
import argparse, hashlib, json, os, subprocess
from pathlib import Path
SCOPE=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
ENV={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
assert ENV['TMPDIR'] and not ENV['TMPDIR'].startswith('/tmp')
assert ENV['CARGO_TARGET_DIR'] and ENV['GLASSMIND_SCRATCH']
def run(cmd,root):
 p=subprocess.run(SCOPE+list(map(str,cmd)),cwd=root,env=ENV,text=True,capture_output=True,timeout=620)
 return dict(code=p.returncode,stdout=p.stdout,stderr=p.stderr)
def sha(s): return hashlib.sha256(s.encode()).hexdigest()
CASES=[]
def add(id, owner='Parcel',other='Ledger',old='count',new='total',a=3,b=9,shape='direct',supported=True,request=None):
 decl=f'pub struct {owner} {{ pub {old}: u32 }}\npub struct {other} {{ pub {old}: u32 }}\n'
 left=f'{owner} {{ {old}: {a} }}.{old}'
 right=f'{other} {{ {old}: {b} }}.{old}'
 prefix=''
 if shape in ('typed','reference','shadow','call','alias','closure','destructure','shorthand'):
  prefix=f'fn read(x: {"&" if shape=="reference" else ""}{owner}) -> u32 {{ x.{old} }}\n'
  left=f'read({"&" if shape=="reference" else ""}{owner} {{ {old}: {a} }})'
 if shape=='shadow': prefix=f'fn read(x: {owner}) -> u32 {{ let x = {other} {{ {old}: x.{old} }}; x.{old} }}\n'
 if shape=='call': prefix=f'fn make() -> {owner} {{ {owner} {{ {old}: {a} }} }}\n';left=f'make().{old}'
 if shape=='alias': prefix=f'type Alias = {owner};\n'; left=f'Alias {{ {old}: {a} }}.{old}'
 if shape=='closure': prefix=f'fn read(x: {owner}) -> u32 {{ (|x: {other}| x.{old})({other} {{ {old}: x.{old} }}) }}\n'
 if shape=='destructure': prefix=f'fn read(x: {owner}) -> u32 {{ let {owner} {{ {old}: n }} = x; n }}\n'
 if shape=='shorthand': prefix=f'fn read(x: {owner}) -> u32 {{ let {old} = x.{old}; {owner} {{ {old} }}.{old} }}\n'
 if shape=='macro': prefix=f'macro_rules! get {{ ($x:expr) => {{ $x.{old} }} }}\n';left=f'get!({owner} {{ {old}: {a} }})'
 if shape=='generic': prefix=f'fn unused<{owner}>(x: {owner}) -> {owner} {{ x }}\n'
 source=decl+prefix+f'pub fn observe() -> (u32,u32) {{ ({left}, {right}) }}\n'
 expected=source.replace(f'pub struct {owner} {{ pub {old}:',f'pub struct {owner} {{ pub {new}:').replace(f'{owner} {{ {old}:',f'{owner} {{ {new}:').replace(f'{owner} {{ {new}: {a} }}.{old}',f'{owner} {{ {new}: {a} }}.{new}')
 if shape in ('typed','reference'): expected=expected.replace(f'x.{old}',f'x.{new}')
 if shape=='nested':
  source='pub mod model {\n'+source+'}\npub fn observe() -> (u32,u32) { model::observe() }\n'
  expected='pub mod model {\n'+expected+'}\npub fn observe() -> (u32,u32) { model::observe() }\n'
 tail=f'// {old} stays in prose\npub const NOTE: &str = "{old}";\n#[test] fn behavior() -> Result<(), String> {{ println!("OWNER_BEHAVIOR_{id}"); if observe() == ({a},{b}) {{ Ok(()) }} else {{ Err("behavior changed".into()) }} }}\n'
 source+=tail;expected+=tail
 if shape=='collision': source+=f'pub const {new}: u32 = 99;\n';expected=source
 CASES.append(dict(id=id,source=source,expected=expected if supported else source,request=request or f'rename field `{old}` on `{owner}` to `{new}`',supported=supported))
add('direct');add('typed',owner='CrateBox',other='Account',old='amount',new='quantity',a=17,b=41,shape='typed');add('reference',owner='Vessel',other='Bucket',old='level',new='height',a=6,b=29,shape='reference');add('nested',owner='Metric',other='Counter',old='value',new='reading',a=11,b=33,shape='nested')
for shape in ['shadow','call','alias','closure','destructure','shorthand','macro','generic','collision']: add(shape,shape=shape,supported=False)
add('bare',supported=False,request='rename field `count` to `total`')
add('missing',supported=False,request='rename field `count` on `Missing` to `total`')
add('ambiguous_owner',owner='Parcel',other='Ledger',supported=False,request='rename field `count` on struct to `total`')
# Freeze all authored contracts before implementation. Sample is the first three.
def main():
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--sample',action='store_true');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
 rows=[]
 for c in CASES[:3] if a.sample else CASES:
  root=(a.output/c['id']).resolve();(root/'src').mkdir(parents=True,exist_ok=True)
  manifest='[package]\nname="owner_case"\nversion="0.0.0"\nedition="2021"\n[workspace]\n';(root/'Cargo.toml').write_text(manifest);(root/'src/lib.rs').write_text(c['source'])
  (a.output/(c['id']+'.before.rs')).write_text(c['source']);(a.output/(c['id']+'.expected.rs')).write_text(c['expected'])
  cmd=['cargo','test','-j1','--','--test-threads','4','--nocapture']
  before=run(cmd,root);edit=run([a.binary.resolve(),'do',root,c['request']],root);actual=(root/'src/lib.rs').read_text();after=run(cmd,root)
  (a.output/(c['id']+'.after.rs')).write_text(actual)
  behavior=all(r['code']==0 and '1 passed' in r['stdout'] and f'OWNER_BEHAVIOR_{c["id"]}' in r['stdout'] for r in [before,after])
  row=dict(id=c['id'],supported=c['supported'],changed=actual!=c['source'],exact=actual==c['expected'],behavior=behavior,source_sha=sha(c['source']),actual_sha=sha(actual),expected_sha=sha(c['expected']),before=before,edit=edit,after=after)
  row['pass']=row['exact'] and behavior and (root/'Cargo.toml').read_text()==manifest
  rows.append(row);print(c['id'],row['changed'],row['exact'],behavior,repr(edit['stdout']),flush=True)
 (a.output/'results.json').write_text(json.dumps(rows,indent=2)+'\n')
 print('correct',sum(r['pass'] for r in rows),'/',len(rows),'edits',sum(r['changed'] for r in rows))
if __name__=='__main__': main()
