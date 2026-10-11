import json
from pathlib import Path
cases=[]
def add(q,v=None,u=None):cases.append(dict(question=q,value=v,unit=u))
# Independent handwritten reciprocals: 1/6+1/3=1/2; 1/12+1/6=1/4.
for a,b in [('Mira','Noah'),('X','Y')]:
 add(f'{a} can complete a job in 6 hours. {b} can finish the same job in 3 hours. How long will {a} and {b} take to complete the job together?', '2','hours')
 add(f'{b} takes 3 hours to finish a job. {a} takes 6 hours to complete the same job. In how many hours can {a} and {b} together do the job?', '2','hours')
 add(f'{a} and {b} together can do a piece of work in 4 days. {a} alone can do the same work in 12 days. How long will {b} take to complete the work alone?', '6','days')
 add(f'{a} alone takes 12 days to complete a job. {a} and {b} together take 4 days to finish the same job. In how many days can {b} alone finish the job?', '6','days')
add('Arun can do a job in 6 hours and Bela can do the same job in 3 hours. How long will Arun and Bela take to do the job together?','2','hours')
add('Arun and Bela can complete a job in 6 and 3 hours respectively. In how many hours can Arun and Bela together complete the job?','2','hours')
add('Arun can complete a job in 6 hours. Bela can complete it in 3 hours. Cleo can finish the same job in 2 hours. How long will Arun, Bela and Cleo take to finish the job together?','1','hours')
add('Cleo takes 2 hours to finish a job; Bela takes 3 hours to complete it; Arun takes 6 hours to do the same work. In how many hours can Arun, Bela and Cleo together finish the job?','1','hours')
add('Arun and Bela and Cleo together can do a job in 1 hour. Arun can do the same job in 6 hours. Bela can do it in 3 hours. In how many hours can Cleo alone do the job?','2','hours')
add('Pipe p can fill a tank in 6 hours. Pipe q can fill the same tank in 3 hours. The tank is empty. How long will p and q take to fill the tank together?','2','hours')
add('Pipe q takes 3 hours to fill a tank. Pipe p takes 6 hours to fill it. The tank is initially empty. In how many hours can p and q together fill the tank?','2','hours')
add('Pipe p can fill a tank in 3 hours. Pipe q can empty it in 6 hours. The tank is empty. How long will p and q take to fill the tank together?','6','hours')
add('Pipe p can fill a tank in 6 hours. Pipe q can drain the same tank in 3 hours. The tank is full. In how many hours can p and q together empty the tank?','6','hours')
add('Pipe p takes 6 hours to empty a tank. Pipe q takes 3 hours to drain it. The tank is initially full. How long will p and q take to empty the tank together?','2','hours')
add('Mira can do a job in 120 minutes. Noah can finish the same work in 3 hours. In how many minutes can Mira and Noah together finish the job?','72','minutes')
add('Noah takes 180 minutes to finish a job. Mira takes 2 hours to complete it. In how many hours can Mira and Noah together finish the job?','6/5','hours')
add('Mira alone can do a job in 12 hours. Mira and Noah together can finish it in 4 hours. How long will Noah take to finish the same job alone?','6','hours')
add('Mira can do a job in 6 days. Noah can do the same work in 3 days. If Mira and Noah work together, how long will they take to complete the job?','2','days')
add('Mira takes 6 days to do a job. Noah takes 3 days to finish it. If both work together, in how many days can they complete the job?','2','days')
add('Mira can do a job in 1.5 hours. Noah can do it in 3 hours. In how many minutes can Mira and Noah together do the job?','60','minutes')
base='Mira can do a job in 6 hours. Noah can finish the same job in 3 hours. '
for q in ['How long will Mira and Zoe take to do the job together?',
 'Mira can do the same job in 7 hours. How long will Mira and Noah take to do the job together?',
 'Noah leaves after 1 hour. How long will Mira and Noah take to do the job together?',
 'Mira slows down. How long will Mira and Noah take to do the job together?',
 'How long will Mira take to do the job alone?',
 'How long will Mira and Noah take to do another job together?',
 'How long will Mira and Noah take to do the job together? Add 3 hours.',
 'How long will Mira and Noah and Zoe take to do the job together?']:
 add(base+q)
for q in ['Pipe p can fill a tank in 3 hours. Pipe q can empty it in 6 hours. How long will p and q take to fill the tank together?',
 'Pipe p can fill a tank in 6 hours. Pipe q can empty it in 3 hours. The tank is empty. How long will p and q take to fill the tank together?',
 'Pipe p can fill a tank in 6 hours. Pipe q can fill it in 3 hours. The tank is half full. How long will p and q take to fill the tank together?',
 'Pipe p can fill a 100 liter tank in 6 hours. Pipe q can fill it in 3 hours. The tank is empty. How long will p and q take to fill the tank together?',
 'Mira can do a job in 0 hours. Noah can do it in 3 hours. How long will Mira and Noah take to do the job together?',
 'Mira and Noah together can do a job in 12 hours. Mira can do it in 6 hours. How long will Noah take to do the job alone?',
 'Mira can do a job in 6 hours. Noah can do a different job in 3 hours. How long will Mira and Noah take to do the job together?',
 'Mira and Noah together can do a job in 3 hours. How long will Noah take to do the job alone?']:
 add(q)
Path(__file__).with_name('authored.json').write_text(json.dumps(cases,indent=2)+'\n')
