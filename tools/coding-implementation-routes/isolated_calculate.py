#!/usr/bin/env python3
"""Supplemental isolated cases; does not replace the failed frozen boundary gate."""
import pathlib,subprocess,os,json,sys,hashlib
binary,out=sys.argv[1:];out=pathlib.Path(out);out.mkdir(parents=True,exist_ok=False);rows=[]
for name in ['implement','implemented']:
 for kind in ['query','rename']:
  root=out/(name+'-'+kind);(root/'src').mkdir(parents=True)
  code=f'pub fn {name}() -> i32 {{ 3 }}\npub fn invoke() -> i32 {{ {name}() }}\n'
  (root/'src/lib.rs').write_text(code);(root/'Cargo.toml').write_text('[package]\nname="isolated"\nversion="0.1.0"\nedition="2021"\n')
  text=f'which functions call {name}?' if kind=='query' else f'rename {name} to calculate'
  env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
  r=subprocess.run([binary,'do',str(root),text],env=env,text=True,capture_output=True,timeout=110)
  after=(root/'src/lib.rs').read_text();ok=('invoke' in r.stdout and code==after) if kind=='query' else ('fn calculate()' in after and 'calculate() }' in after)
  rows.append(dict(text=text,ok=ok,stdout=r.stdout,before=hashlib.sha256(code.encode()).hexdigest(),after=hashlib.sha256(after.encode()).hexdigest()))
(out/'results.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2))
