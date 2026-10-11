"""Independent verification of retained accepted outputs, never candidate input."""
import pathlib,json,os,subprocess,shutil,sys,re
R=pathlib.Path(sys.argv[1]).resolve();tag=sys.argv[2];out=R/(tag+'-verification');out.mkdir(exist_ok=False);rows=[]
for i,r in enumerate(json.loads((R/tag/'results.json').read_text())):
 if not r['accepted']:continue
 root=R/tag/r['id'].replace('/','_').replace('#','_');copy=out/str(i);shutil.copytree(root,copy)
 env=dict(os.environ);env['CARGO_TARGET_DIR']=str(pathlib.Path(env['CARGO_TARGET_DIR'])/'independent'/tag/str(i))
 p=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4'],cwd=copy,env=env,capture_output=True,text=True,timeout=620)
 (out/f'{i}.log').write_text(p.stdout+p.stderr)
 evidence=pathlib.Path(r['stdout'].split('Evidence: ',1)[1].strip());choice=json.loads((evidence/'choice.json').read_text());logs=[(evidence/f'candidate-{j}.log').read_text() for j in range(choice['candidate_count'])]
 passing=[j for j,log in enumerate(logs) if log.startswith('status=exit status: 0 ')]
 ok=p.returncode==0 and '1 passed' in p.stdout and passing==choice['passing_indices'] and len(passing)==1 and 'glassmind_target_executed_' in (evidence/'target-execution.log').read_text() and '1 failed' in (evidence/'baseline.log').read_text() and (evidence/'apply.log').read_text().startswith('status=exit status: 0 ')
 rows.append(dict(id=r['id'],ok=ok,evidence=str(evidence),choice=choice,independent_exit=p.returncode))
(out/'results.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2));assert all(x['ok'] for x in rows)
