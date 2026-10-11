"""Replay question-only ordinary facts/default talk; preserve full dev denominator."""
from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('binary');p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(exist_ok=False)
r=Path('/home/gsc/.cache/glassmind/continuous-20261010');bundle=r/'factsets-run/tmp/sets/bundle';base=r/'coordinator/factsets-check/regression'
rows=[l.split('\t',3)[3] for l in Path('/home/gsc/.cache/glassmind/data/simplequestions/valid.tsv').read_text().splitlines()];assert len(rows)==2821
report=[]
for n in [200,2821]:
 data=''.join('/reset\n'+q+'\n' for q in rows[:n]).encode()
 for mode,extra in [('talk',['--facts-dir',str(bundle)]),('default',[])]:
  result=subprocess.run([a.binary,'talk','--json',*extra],input=data,capture_output=True,check=True,timeout=550).stdout
  name=f'{mode}-{n}.jsonl';(a.out/name).write_bytes(result)
  same=result==(base/name).read_bytes();report.append(dict(mode=mode,questions=n,bytes=len(result),sha256=hashlib.sha256(result).hexdigest(),baseline_byte_identical=same))
(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));assert all(x['baseline_byte_identical'] for x in report)
