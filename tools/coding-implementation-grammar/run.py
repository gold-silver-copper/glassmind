import pathlib,json,hashlib,subprocess,os,sys,shutil
T=pathlib.Path(__file__).resolve().parent;R=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run');S=R.parent/'implgrammar-source'
mode,binary,tag=sys.argv[1:];out=R/tag;out.mkdir(parents=True,exist_ok=False)
def snap(p):return {str(f.relative_to(p)):hashlib.sha256(f.read_bytes()).hexdigest() for f in p.rglob('*') if f.is_file()}
rows=json.loads((T/('boundaries.json' if mode=='boundaries' else 'native.json')).read_text())
if mode.startswith('train'):rows=json.loads((S/'requests.json').read_text());rows=[x for x in rows if x['source']=='rb-implement-sum-even'] if mode=='train-sample' else rows
if mode=='sample':rows=rows[:3]
results=[]
for i,row in enumerate(rows):
 root=out/(row['id'].replace('/','_').replace('#','_'));(root/'src').mkdir(parents=True)
 if 'source' in row:
  for rel in ['Cargo.toml','src/lib.rs']:shutil.copy(S/'sources'/row['source']/rel,root/rel)
 else:
  (root/'Cargo.toml').write_text('[package]\nname="grammar_contract"\nversion="0.1.0"\nedition="2021"\n[workspace]\n');(root/'src/lib.rs').write_text(row['code'])
 before=snap(root);env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(out/'scratch')
 if mode=='baseline-native':
  if not row['accept']:continue
  env['CARGO_TARGET_DIR']=str(pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'baseline-contracts'/str(i))
  cmd=['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4']
 else:cmd=[binary,'do',str(root),row['text']]
 p=subprocess.run(cmd,cwd=root,env=env,capture_output=True,text=True,timeout=620)
 (out/(root.name+'.log')).write_text(p.stdout+p.stderr);after=snap(root);accepted='Applied test-verified proposal' in p.stdout
 ok=True
 if mode=='baseline-native':ok=p.returncode!=0 and '1 failed' in p.stdout and 'could not compile' not in p.stderr
 elif mode!='boundaries':
  ok=(accepted==row['accept'] if 'accept' in row else True) and (before==after if not accepted else set(before)==set(after) and all(before[k]==after[k] for k in before if k!='src/lib.rs'))
  if accepted:
   original=row.get('code') or (S/'sources'/row['source']/'src/lib.rs').read_text();marker='unimplemented!()' if 'unimplemented!()' in original else 'todo!()';l,r=original.split(marker,1);now=(root/'src/lib.rs').read_text();ok=ok and now.startswith(l) and now.endswith(r)
 result=dict(id=row['id'],text=row['text'],ok=ok,accepted=accepted,before=before,after=after,stdout=p.stdout.replace(str(root),'ROOT'),exit=p.returncode)
 results.append(result);(out/'results.json').write_text(json.dumps(results,indent=2,ensure_ascii=False)+'\n');print(row['id'],ok,accepted,flush=True)
if mode=='boundaries' and not tag.startswith('baseline-boundaries'):
 baseline=json.loads((R/'baseline-boundaries-complete/results.json').read_text())
 for a,b in zip(results,baseline):a['ok']=all(a[k]==b[k] for k in ['before','after','stdout','exit'])
 (out/'results.json').write_text(json.dumps(results,indent=2,ensure_ascii=False)+'\n')
raise SystemExit(0 if all(x['ok'] for x in results) else 1)
