"""Run the unchanged existing joint diagnostic, with full dev denominator."""
from pathlib import Path
import subprocess,sys
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');R=W/'timedmatch-run';O=R/'evidence'
q=O/'questions.txt';q.write_text(''.join(row.split('\t',3)[3]+'\n' for row in (W/'factmatch-source/valid.tsv').read_text().splitlines()))
for phase in ['baseline','current']:
    subprocess.run([sys.executable,'tools/facts/joint-audit.py','--labels',str(W/'factmatch-source/valid.tsv'),'--questions',str(q),'--predictions',str(O/phase/'predictions-2821.jsonl'),'--official-totals',str(O/phase/'score-2821.json'),'--scope','Existing 500 validation-subject-seeded cache, full public dev','--output',str(O/phase/'joint.json')],check=True)
