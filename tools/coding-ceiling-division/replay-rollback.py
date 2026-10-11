"""Confirm the rounding candidate fails native assertions without touching the original."""
import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
run=Path(sys.argv[1]).resolve(); original=run/'authored-after/sources/rollback'; target=run/'rollback-replay'
shutil.copytree(original,target,dirs_exist_ok=True)
p=target/'src/lib.rs'; old=p.read_text(); new=old.replace('n / 3','::core::primitive::u32::div_ceil(n, 3)',1); assert new!=old; p.write_text(new)
env={k:v for k,v in os.environ.items() if not k.endswith('API_KEY')}
with (run/'rollback-native.log').open('w') as log:
 r=subprocess.run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','-j1','--','--test-threads','4'],cwd=target,env=env,stdout=log,stderr=subprocess.STDOUT)
assert r.returncode!=0 and (original/'src/lib.rs').read_text()==old
(run/'rollback-proof.json').write_text(json.dumps(dict(native_returncode=r.returncode,original_sha256=hashlib.sha256(old.encode()).hexdigest(),candidate_sha256=hashlib.sha256(new.encode()).hexdigest(),original_unchanged=True),indent=2)+'\n')
