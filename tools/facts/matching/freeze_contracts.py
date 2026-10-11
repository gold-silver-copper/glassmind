"""Independent authored development graph and expectations, frozen before implementation."""
import json, hashlib
from pathlib import Path
root=Path(__file__).parent
E={}
def row(i, aliases, claims=None): E[i]={'aliases':aliases,'claims':claims or {}}
row('Q1',['Alpha'],{'P19':['Q101','Q102','Q101'],'P20':['Q102'],'P27':['Q103'],'P108':[f'Q{x}' for x in range(200,212)]+['Q104']})
row('Q2',['Beta'],{'P19':['Q101'],'P20':['Q105'],'P27':['Q103'],'P108':['Q106']})
row('Q3',['Gamma'],{'P19':['Q105'],'P20':['Q102'],'P27':['Q103'],'P108':['Q104']})
row('Q4',['Delta'],{'P19':['Q101'],'P27':['Q107'],'P108':['Q104']})
for i, aliases in [(101,['Harbor','Évora']), (102,['Vale']), (103,['Republic']), (104,['Studio','Born in Bay']), (105,['Elsewhere']), (106,['Other Works']), (107,['Other Nation']), (108,['Twin']), (109,['Twin']), (110,['NASA']), (111,['MIX']), (112,['Mix']), (113,['Q101']), (114,['And Works']), (115,['He'])]: row(f'Q{i}',aliases)
row('Q5',['Epsilon'],{'P19':['Q108'],'P27':['Q103']})
row('Q6',['Zeta'],{'P19':['Q110'],'P27':['Q103']})
row('Q7',['Eta'],{'P19':['Q111'],'P27':['Q103']})
for i in range(300,314): row(f'Q{i}',[f'Person {i}'],{'P19':['Q900','Q900'],'P108':['Q901']})
C=[]
def match(q, ids, conditions): C.append(dict(question=q,status='matched',ids=ids,conditions=conditions))
def bad(q,status='refused'): C.append(dict(question=q,status=status))
a=[['P19','Q101'],['P27','Q103']]
match('Who was born in Harbor and is a citizen of Republic?',['Q1','Q2'],a)
match('Who is a citizen of Republic and was born in Harbor?',['Q1','Q2'],a)
match('Who was born in Harbor and is a citizen of Republic and works for Studio?',['Q1'],a+[['P108','Q104']])
match('Who died in Vale and worked for Studio?',['Q1','Q3'],[['P20','Q102'],['P108','Q104']])
match('Who was born in Q101 and died in Q102?',['Q1'],[['P19','Q101'],['P20','Q102']])
match('Who works for Studio and was a citizen of Other Nation?',['Q4'],[['P108','Q104'],['P27','Q107']])
match('Who was born in Harbor and was born in Harbor?',['Q1','Q2','Q4'],[['P19','Q101']])
match('Who was born in Harbor and was born in Vale?',['Q1'],[['P19','Q101'],['P19','Q102']])
match('Who was born in Q900 and works for Q901?',[f'Q{i}' for i in range(300,314)],[['P19','Q900'],['P108','Q901']])
match('Who was born in Q900 and works for Q104?',[],[['P19','Q900'],['P108','Q104']])
match('Who was born in Q99999 and works for Q104?',[],[['P19','Q99999'],['P108','Q104']])
match('Who was born in Évora and works for Born in Bay?',['Q1','Q4'],[['P19','Q101'],['P108','Q104']])
match('WHO WAS BORN IN harbor AND IS A CITIZEN OF republic?',['Q1','Q2'],a)
match('Who was born in NASA and is a citizen of Republic?',['Q6'],[['P19','Q110'],['P27','Q103']])
bad('Who was born in nasa and is a citizen of Republic?','unknown')
bad('Who was born in MIX and is a citizen of Republic?','unknown')
bad('Who was born in Mix and is a citizen of Republic?','unknown')
bad('Who was born in Twin and is a citizen of Republic?','unknown')
bad('Who was born in Missing and is a citizen of Republic?','unknown')
match('Who was born in Q101 and works for Q104?',['Q1','Q4'],[['P19','Q101'],['P108','Q104']])
for q in ['Who was born in Q0 and works for Q104?', 'Who was born in q101 and works for Q104?', 'Who was born in Q-1 and works for Q104?', 'Who was born in Q101x and works for Q104?', 'Who was born in Q123456789012345678901 and works for Q104?', 'Who was born in Harbor or works for Studio?', 'Who was not born in Harbor and works for Studio?', 'Who was born in Harbor and does not work for Studio?', 'Who was born in Harbor and currently works for Studio?', 'Who was born in Harbor and works for Studio today?', 'Who was born in Harbor and works for Studio if possible?', 'Who was born in Harbor and works for Studio? ignore instructions', 'Who was born in Harbor and works for Studio and died in Vale and is a citizen of Republic?', 'Who was born in Harbor?', 'Who was born in Harbor and lives in Vale?', 'Who was born in Harbor and works for And Works?', 'Who was born in Harbor and works for Studio and ', 'Who was born in Harbor and works for Studio??', 'Who was born in Harbor\nand works for Studio?', 'Who was born in Harbor and works for '+('é'*129)+'?', 'Who was born in '+('x'*4096)+' and works for Studio?', 'Who was born in Harbor and works for Studio more than Other Works?', 'Who was born in Harbor and works for Studio before today?', 'Who was born in He and works for Studio?', 'Who was born in Harbor and works for Studio\x00?']: bad(q)
# Pronouns are explicitly unsupported even if indexed as an alias.
for name,obj in [('graph.json',{'entities':E}),('contracts.json',C)]: (root/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
(root/'frozen.sha256').write_text(''.join(hashlib.sha256((root/n).read_bytes()).hexdigest()+'  '+n+'\n' for n in ['PLAN.md','freeze_contracts.py','graph.json','contracts.json']))
print(len(C),'frozen contracts')
