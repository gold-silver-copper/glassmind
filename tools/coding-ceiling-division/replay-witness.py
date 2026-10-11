"""Replay both recorded variants against visible tests and distinct native outputs."""
import json,os,subprocess,sys
from pathlib import Path
witness=Path(sys.argv[1]); out=Path(sys.argv[2]).resolve(); out.mkdir(parents=True,exist_ok=True)
r=json.loads(witness.read_text()); w=r['witness']; records=[]
env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
for side in ['first','second']:
 root=out/side; (root/'src').mkdir(parents=True,exist_ok=True)
 (root/'Cargo.toml').write_text('[package]\nname="witness_'+side+'"\nversion="0.0.0"\nedition="2024"\n[workspace]\n')
 text=r[side+'_source']
 text+='\n#[cfg(test)] mod independent_replay { #[test] fn native_output() { assert_eq!(super::'+w['function']+'('+w['input']+'), '+w[side+'_output']+'); println!("REPLAY_'+side+'"); } }\n'
 (root/'src/lib.rs').write_text(text)
 with (out/(side+'.log')).open('w') as log:
  status=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--nocapture','--test-threads','4'],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
 assert status==0 and 'REPLAY_'+side in (out/(side+'.log')).read_text()
 records.append(dict(side=side,input=w['input'],output=w[side+'_output'],status=status))
assert records[0]['output'] != records[1]['output']
(out/'report.json').write_text(json.dumps(records,indent=2)+'\n')
