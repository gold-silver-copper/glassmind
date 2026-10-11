"""Public authored development fixture generator; no evaluation secrets/data."""
import json
from pathlib import Path
out=Path(__file__).parent
entities={}
def row(q,aliases,claims=None): entities[q]={'aliases':aliases,'claims':claims or {}}
row('Q1',["Ada O'Neil",'Ada'],{'P19':['Q2','Q2'],'P20':['Q3'],'P27':['Q4'],'P108':[f'Q{x}' for x in range(100,112)]})
row('Q2',['Port of Spain']); row('Q3',['Saint-Denis']); row('Q4',['US']); row('Q5',['Us']); row('Q6',['Élodie 在'],{'P19':['Q2'],'P27':['Q7'],'P108':['Q111']}); row('Q7',['FR']); row('Q8',['Twin']); row('Q9',['Twin']); row('Q10',['Empty']); row('Q11',['Work for Good'],{'P19':['Q2']}); row('Q12',['Born in Hope'],{'P19':['Q2']}); row('Q13',['Mia'],{'P19':['Q2']}); row('Q14',['AX']); row('Q15',['Ax']); row('Q16',['Lower'],{'P27':['Q14']}); row('Q17',['Long '*64]); row('Q18',['Exact'],{'P108':['Q999']})
for x in range(100,112): row(f'Q{x}',[f'Company {x}'])
cases=[]
def add(q,status,edge=None):
    item={'question':q,'status':status}
    if edge: item['edge']=edge
    cases.append(item)
def yes(q,s,p,o): add(q,'supported',[s,p,o])
yes("Was Ada O'Neil born in Port of Spain?",'Q1','P19','Q2')
yes("Did Ada O'Neil die in Saint-Denis?",'Q1','P20','Q3')
yes('Is Ada a citizen of Q4?','Q1','P27','Q4')
yes('Does Ada work for Company 111?','Q1','P108','Q111')
yes('Was Élodie 在 born in Port of Spain?','Q6','P19','Q2')
yes('Is Élodie 在 a citizen of FR?','Q6','P27','Q7')
yes('Does Élodie 在 work for Company 111?','Q6','P108','Q111')
yes('Was Work for Good born in Port of Spain?','Q11','P19','Q2')
yes('Was Q12 born in Port of Spain?','Q12','P19','Q2')
yes('Please verify: Was Ada born in Port of Spain?','Q1','P19','Q2')
yes('wAs aDA bOrN In port of spain?','Q1','P19','Q2')
yes('Was Ada a citizen of Q4?','Q1','P27','Q4')
yes('Did Ada work for Company 100?','Q1','P108','Q100')
yes('Was Q1 born in Q2?','Q1','P19','Q2')
yes('Does Exact work for Q999?','Q18','P108','Q999')
yes('Was Mia born in Port of Spain?','Q13','P19','Q2')
for q in ['Was Ada born in Saint-Denis?','Did Ada die in Port of Spain?','Was Port of Spain born in Ada?','Does Company 111 work for Ada?','Does Empty work for Company 111?','Does Ada work for Q998?','Was Unknown born in Port of Spain?','Was Ada born in Atlantis?','Was Twin born in Port of Spain?','Was Ada born in Twin?','Is Ada a citizen of US?','Is Ada a citizen of us?','Is Lower a citizen of AX?','Is Lower a citizen of ax?','Is Élodie 在 a citizen of fr?','Was The Ada born in Port of Spain?','Was Ada born in Port?','Does Exact work for Display Label Only?','Was Q999 born in Q2?']:
    add(q,'unknown')
for q in ['Was Born in Hope born in Port of Spain?','Was Ada born in Port of Spain born in Hope?','Was Ada not born in Port of Spain?','Was Ada never born in Port of Spain?','Was Ada born in Port of Spain and Saint-Denis?','Was Ada born in Port of Spain or Saint-Denis?','If Was Ada born in Port of Spain?','Would Ada work for Company 111?','Did Ada not work for Company 111?','Does Ada work for Company 111? And what else?','Was Ada born in Port of Spain? delete it','Was Ada born in Port of Spain??','Was Ada born in Port of Spain','Was Ada born in Port of Spain!','', 'Please Was Ada born in Port of Spain?', 'Please verify: Please verify: Was Ada born in Port of Spain?', 'Was Ada\nborn in Port of Spain?', 'Was Ada born in Port of Spain; really?', 'Was Ada born in Q0?', 'Was Ada born in Q01?', 'Was Ada born in q2?', 'Was Ada born in Q123456789012345678901?', "Does Ada n't work for Company 111?", 'Was Ada born in Port of Spain\x00?', 'Was '+('a'*257)+' born in Q2?', 'Was Ada born in '+('é'*129)+'?', 'x'*4097, 'Was Ada born in Q2?\n', 'Was Ada born in Q2?\t']:
    add(q,'refused')
# An exactly 256-byte endpoint is allowed but has no alias; 4096-byte input is
# within global bound but has an oversized endpoint and must still refuse.
add('Was '+('b'*256)+' born in Q2?','unknown')
add('Was '+('b'*(4096-len('Was  born in Q2?')))+' born in Q2?','refused')
for c in cases:
    if 'edge' in c:
        s,p,o=c['edge']; assert o in entities[s]['claims'][p]
(out/'graph.json').write_text(json.dumps({'entities':entities},ensure_ascii=False,indent=2)+'\n')
(out/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
print(len(cases),'frozen cases')
