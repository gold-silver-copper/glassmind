#!/usr/bin/env python3
"""Run unchanged visible legacy dev fixtures, optionally injecting facts at process boundary."""
import argparse, json, subprocess, sys
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--facts-dir',type=Path);p.add_argument('--model',type=Path);p.add_argument('--output',type=Path,required=True)
a=p.parse_args(); a.output.mkdir(parents=True,exist_ok=True)
# A local executable wrapper preserves the existing fixture scripts and scorers.
wrapper=a.output/'talk-wrapper'
extra=['--facts-dir',str(a.facts_dir.resolve())] if a.facts_dir else []
wrapper.write_text('#!/usr/bin/env python3\nimport os,sys\nargs=sys.argv[1:]\nos.execv('+repr(str(a.binary.resolve()))+', ['+repr(str(a.binary.resolve()))+']+args[:1]+'+repr(extra)+'+args[1:])\n')
wrapper.chmod(0o755)
model=['--model',str(a.model)] if a.model else []
commands=[
 [sys.executable,'crates/everyday/tools/smoke_memory.py',str(wrapper),str(a.output/'memory'),'--command','talk'],
 [sys.executable,'tools/talk-dev.py',str(wrapper),'--output',str(a.output/'calculations.json'),*model],
 [sys.executable,'tools/talk-story-dev.py',str(wrapper),'--mode','mixed','--output',str(a.output/'stories.json'),*model],
]
for name,command in zip(['memory','calculations','stories'],commands):
    result=subprocess.run(command,text=True,capture_output=True,timeout=550)
    (a.output/(name+'.stdout')).write_text(result.stdout)
    (a.output/(name+'.stderr')).write_text(result.stderr)
    print(name,result.returncode,result.stdout)
    result.check_returncode()
assert json.loads((a.output/'calculations.json').read_text())['right']==27
assert json.loads((a.output/'stories.json').read_text())['totals']=={'right':47}
