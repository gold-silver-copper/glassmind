"""Sequential bounded offline reproduction; no cargo, models or services."""
import argparse
import os
from pathlib import Path
import subprocess
import sys

HERE=Path(__file__).resolve().parent
WORK=HERE.parents[3]
p=argparse.ArgumentParser()
p.add_argument('--data',type=Path,default=WORK/'quantities-run/tmp/mathqa')
p.add_argument('--deps',type=Path,default=WORK/'quantities-run/tmp/quantities-deps')
p.add_argument('--frozen',type=Path,default=WORK/'typedmetric-run/baseline')
p.add_argument('--output',type=Path,default=HERE/'reproduced')
a=p.parse_args()
env=os.environ.copy()
env['PYTHONDONTWRITEBYTECODE']='1'; env['PYTHONPATH']=str(a.deps)
subprocess.run([sys.executable,'-m','unittest','discover','-s',str(HERE),'-p','test_metric.py','-v'],env=env,check=True,timeout=60)
for name,extra in [('sample',['--sample']),('full',[])]:
    subprocess.run([sys.executable,str(HERE/'replay.py'),'--data',str(a.data),'--frozen',str(a.frozen),
                    '--output',str(a.output/name),*extra],env=env,check=True,timeout=120)
