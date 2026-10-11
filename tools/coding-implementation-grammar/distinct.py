"""Execute the two retained ambiguity survivors on a fresh authored contrast."""
import pathlib,json,shutil,subprocess,os,re,hashlib
R=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run');out=R/'distinct-witness';out.mkdir(exist_ok=False)
row=next(r for r in json.loads((R/'final-native/results.json').read_text()) if r['id']=='ambiguous');evidence=pathlib.Path(row['stdout'].split('evidence: ')[1].strip());choice=json.loads((evidence/'choice.json').read_text());assert choice['candidate_count']==2 and choice['passing_indices']==[0,1]
results=[]
for i in choice['passing_indices']:
 root=out/str(i);(root/'src').mkdir(parents=True);shutil.copy(R/'final-native/ambiguous/Cargo.toml',root/'Cargo.toml');original=(evidence/f'candidate-{i}.rs').read_bytes();(root/'src/lib.rs').write_bytes(original+b'\n#[cfg(test)] mod contrast { #[test] fn outputs() { println!("OBSERVED:{}",super::target("ab")); } }\n')
 env=dict(os.environ);env['CARGO_TARGET_DIR']=str(pathlib.Path(env['CARGO_TARGET_DIR'])/'distinct'/str(i));p=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4','--nocapture'],cwd=root,env=env,capture_output=True,text=True,timeout=620);(out/f'{i}.log').write_text(p.stdout+p.stderr);value=int(re.search(r'OBSERVED:(\d+)',p.stdout).group(1));assert p.returncode==0
 results.append(dict(candidate=i,sha256=hashlib.sha256(original).hexdigest(),observed=value,exit=p.returncode))
assert {x['observed'] for x in results}=={1,2};(out/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results,indent=2))
