"""Frozen authored development cases; no benchmark annotations."""
import json, subprocess, sys
from pathlib import Path
cases=[]
def add(q,v=None,u=None):cases.append(dict(question=q,expected=v,unit=u))
base='Mira can do a job in 12 hours. Niko can do the same job in 6 hours. '
add(base+'How many hours will they take to finish the job together?','4','hours')
add('A and B can do a piece of work in 12 and 18 days respectively. In how many days will they finish it together?','36/5','days')
add('X, Y and Z can complete a piece of work in 12, 18 and 36 days respectively. Working together, they will complete the same work in how many days?','6','days')
add('Pipe red can fill a tank in 6 hours. Pipe blue can empty the same tank in 12 hours. How many hours will they take to fill the tank together?','12','hours')
add('Mira and Niko together can do a job in 4 hours. Mira can do the same job in 12 hours. How many hours will Niko take to finish the job alone?','6','hours')
add(base+'They work together for 2 hours and then Niko leaves. How many more hours will Mira take to finish the remaining work?','6','hours')
add(base+'They work together for 2 hours and then Niko leaves. What is the total time in hours to finish the job?','8','hours')
add(base+'Mira works alone for 3 hours and then Niko joins. How many more hours will they take to finish the remaining work together?','3','hours')
add(base+'Mira works alone for 3 hours and then Niko joins. What is the total time in hours to finish the job?','6','hours')
add('Lena can do a job in 2 hours. Omar can do the same job in 120 minutes. In how many minutes will they finish it together?','60','minutes')
add('P can do a work in 15 days and Q in 20 days. In how many days will they finish it together?','60/7','days')
add('Two pipes P and Q can fill a cistern in 12 and 24 minutes respectively. If both the pipes are opened simultaneously, how much time will be taken to fill the cistern?','8','minutes')
add(base+'They work together for 5 hours and then Niko leaves. What is the total time in hours to finish the job?','4','hours')
add(base+'They work together for 4 hours and then Niko leaves. How many more hours will Mira take to finish the remaining work?','0','hours')
add('Pipe red can fill a tank in 6 hours. Pipe blue can fill the same tank in 12 hours. Pipe green can empty it in 24 hours. How many hours will they take to fill the tank together?','24/5','hours')
for q in [
base+'How many hours will Zara take to finish the job alone?',
base+'How many hours will they take to finish the job together? How much does it cost?',
base+'Their rates double every hour. How many hours will they take to finish the job together?',
base+'They work together for -2 hours and then Niko leaves. How many more hours will Mira take to finish the remaining work?',
base+'Mira can do the same job in 15 hours. How many hours will they take to finish the job together?',
base+'Mira can do the same job in 12 hours. How many hours will they take to finish the job together?',
base+'They work together for 2 hours and then Niko leaves. How many more hours will Niko take to finish the remaining work?',
'Mira can do a job in 0 hours. Niko can do the same job in 6 hours. How many hours will they take to finish the job together?',
'Mira can do a job in -3 hours. Niko can do the same job in 6 hours. How many hours will they take to finish the job together?',
'Pipe red can fill a tank in 6 hours. Pipe blue can empty the same tank in 6 hours. How many hours will they take to fill the tank together?',
'Pipe red can fill a tank in 12 hours. Pipe blue can empty the same tank in 6 hours. How many hours will they take to fill the tank together?',
'Pipe red can fill a tank in 6 hours. Pipe blue can empty the same tank in 12 hours. The tank is half full. How many hours will they take to fill the tank together?',
'Mira and Niko together can do a job in 4 hours. How many hours will Niko take to finish the job alone?',
'Mira and Niko together can do a job in 12 hours. Mira can do the same job in 6 hours. How many hours will Niko take to finish the job alone?',
'Mira can do a job in 2 weeks. Niko can do the same job in 6 hours. How many hours will they take to finish the job together?',
'Mira can do a job in 12 hours. Niko can do a different job in 6 hours. How many hours will they take to finish the job together?',
base+'They work together for 2 hours and then Zara leaves. How many more hours will Mira take to finish the remaining work?',
base+'They work together for 2 hours. How many hours will they take to finish the job together?',
]:add(q)
if __name__=='__main__':
 r=subprocess.run([sys.argv[1],'--jsonl'],input=''.join(json.dumps(c['question'])+'\n' for c in cases),capture_output=True,text=True,check=True,timeout=180)
 answers=[json.loads(s) for s in r.stdout.splitlines()];assert len(answers)==len(cases)
 for c,a in zip(cases,answers):c.update(actual=a,passed=(a['status']=='refusal' if c['expected'] is None else a['status']=='answer' and a['value']==c['expected'] and a.get('unit')==c['unit']))
 report=dict(total=len(cases),passed=sum(c['passed'] for c in cases),cases=cases)
 Path(sys.argv[2]).write_text(json.dumps(report,indent=2)+'\n');print({k:v for k,v in report.items() if k!='cases'})
 if report['passed']!=report['total']:print(json.dumps([c for c in cases if not c['passed']],indent=2));sys.exit(1)
