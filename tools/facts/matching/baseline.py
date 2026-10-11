import subprocess,json,sys
from pathlib import Path
root=Path(__file__).parent/'run'/'baseline'; data=Path('/home/gsc/.cache/glassmind/continuous-20261010/factmatch-source')
rows=[r.split('\t',3) for r in (data/'valid.tsv').read_text().splitlines()]
for count in (200,2821):
 raw=subprocess.run([root/'continuous_predictions',data/'relation.json',data/'index.json'],input=''.join(r[3]+'\n' for r in rows[:count]).encode(),capture_output=True,check=True).stdout
 (root/f'standalone-{count}.jsonl').write_bytes(raw)
 raw=subprocess.run([root/'glassmind-facts','score','--input',data/'valid.tsv','--model',data/'relation.json','--index',data/'index.json','--limit',str(count)],capture_output=True,check=True).stdout
 (root/f'native-score-{count}.json').write_bytes(raw)
 print(count,raw.decode().strip(),flush=True)
