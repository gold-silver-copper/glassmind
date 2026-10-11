#!/usr/bin/env python3
"""Authored fixture replay; retains original labels and explicit confirmation contract."""
import argparse,hashlib,json,os,pathlib,subprocess
p=argparse.ArgumentParser();p.add_argument('binary');p.add_argument('input');p.add_argument('output');p.add_argument('--sample',action='store_true');p.add_argument('--confirm',action='store_true');a=p.parse_args()
binary=str(pathlib.Path(a.binary).resolve());out=pathlib.Path(a.output).resolve();out.mkdir(parents=True,exist_ok=True)
env=dict(os.environ);env.pop('CARGO_TARGET_DIR',None);assert env['TMPDIR'] and not env['TMPDIR'].startswith('/tmp')
rows=json.loads(pathlib.Path(a.input).read_text());rows=[r for r in rows if r['group']!='train172']
if a.sample: rows=[r for r in rows if r['group']=='authored' and r['index'] in [0,2,6,7,10,15,17,19,23]]
source='pub fn number()->u32 { 7u8 }\n#[test] fn value(){assert_eq!(number(),7);}\n';fixed=source.replace('7u8','7u32')
def run(cmd,cwd=None):
 r=subprocess.run(cmd,cwd=cwd,env=env,capture_output=True,text=True,timeout=180)
 return dict(exit=r.returncode,stdout=r.stdout,stderr=r.stderr)
def native(w):
 return run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4'],w)
results=[]
for row in rows:
 w=out/(row['group']+'-'+str(row['index']));(w/'src').mkdir(parents=True,exist_ok=True)
 manifest='[package]\nname="veto_'+hashlib.sha256(str(w).encode()).hexdigest()[:16]+'"\nversion="0.1.0"\nedition="2024"\n[workspace]\n'
 (w/'Cargo.toml').write_text(manifest);f=w/'src/lib.rs';f.write_text(source)
 before=native(w)
 preview=run([binary,'ask','--source',str(w),'--json','--apply',row['text']]);preview_source=f.read_text()
 result=run([binary,'do',str(w),'--explain',row['text']]);after=f.read_text();check=native(w)
 positive=row['expected'].startswith('repair')
 rec=dict(row,before=before,preview=preview,preview_source=preview_source,result=result,source_before=source,source_after=after,after=check,
  manifest_unchanged=(w/'Cargo.toml').read_text()==manifest,
  preview_readonly=preview_source==source, exact_patch=after==fixed,unchanged=after==source,
  original_expected_ok=before['exit']!=0 and preview_source==source and after==(fixed if positive else source) and (check['exit']==0)==positive)
 results.append(rec);(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
 print(row['group'],row['index'],rec['original_expected_ok'],flush=True)
 if a.confirm and positive:
  f.write_text(source)
  cmd=[binary,'ask','--source',str(w),'--session',str(w/'session.json'),'--json']
  q=run(cmd+[row['text']]);s=f.read_text();yes=run(cmd+['--apply','yes']);c=native(w)
  rec['confirmation']=dict(preview=q,yes=yes,check=c,ok=s==source and f.read_text()==fixed and c['exit']==0)
  (out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps({'rows':len(results),'original_expected_ok':sum(r['original_expected_ok'] for r in results),'unrequested_edits':sum(not r['expected'].startswith('repair') and not r['unchanged'] for r in results)}))
