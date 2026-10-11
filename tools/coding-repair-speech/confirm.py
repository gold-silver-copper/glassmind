#!/usr/bin/env python3
"""Separate session confirmation protocol; original probe results stay unchanged."""
import hashlib, json, pathlib, subprocess, sys
binary=str(pathlib.Path(sys.argv[1]).resolve()); out=pathlib.Path(sys.argv[2]).resolve(); out.mkdir(parents=True,exist_ok=True)
source='pub fn number()->u32 { 7u8 }\n#[test] fn value(){assert_eq!(number(),7);}\n'
rows=[]
def run(cmd,cwd=None):
 r=subprocess.run(cmd,cwd=cwd,capture_output=True,text=True,timeout=180)
 return dict(exit=r.returncode,stdout=r.stdout,stderr=r.stderr)
def native(w):
 return run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4'],w)
for i,text in enumerate(['Could you fix the build?','Please repair the failing tests.']):
 w=out/str(i);(w/'src').mkdir(parents=True,exist_ok=True);(w/'Cargo.toml').write_text('[package]\nname="confirm_probe"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
 f=w/'src/lib.rs';f.write_text(source); before=native(w)
 base=[binary,'ask','--source',str(w),'--session',str(w/'session.json'),'--json']
 preview=run(base+[text]); preview_source=f.read_text()
 apply=run(base+['--apply','yes']); after=f.read_text(); check=native(w)
 rows.append(dict(text=text,preview=preview,apply=apply,source_before=source,preview_source=preview_source,source_after=after,before=before,after=check,ok=before['exit']!=0 and preview_source==source and after==source.replace('7u8','7u32') and check['exit']==0))
(out/'results.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps({'passed':sum(x['ok'] for x in rows),'total':len(rows)}))
