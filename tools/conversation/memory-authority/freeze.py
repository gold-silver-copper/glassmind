"""Author frozen local development sessions; no upstream evaluation material."""
import json
from pathlib import Path
root=Path(__file__).parent
sessions=[]
def row(text,kind,contains=None,**expected):
    if contains: expected['contains']=contains
    return [text,dict(kind=kind,**expected)]
def add(name,rows,state=None,more=None,initial=None):
    s=dict(name=name,phases=[dict(rows=rows,state=state or {})])
    if more:s['phases']+=more
    if initial:s['initial']=initial
    sessions.append(s)
add('explicit-unicode', [row('My name is 李明','remembered','李明'),row('What is my name?','recalled','李明',source_turn=1),row('Actually, my name is İpek','remembered','İpek'),row('Call me Dr. Neri','remembered','Dr. Neri')], {'name.value':'Dr. Neri','name.stated_on_turn':4,'last_correction.previous.value':'İpek'})
add('name-proposal', [row('My name is Leila','remembered'),row('please change the name you refer to me by',{'model':'refusal','plain':'refusal'}),row("Actually, it's Mara",'missing'),row('What is my name?','recalled','Leila',source_turn=1)],{'name.value':'Leila','name.stated_on_turn':1})
add('ambiguous-call', [row('My name is Sora','remembered'),row('Call me next week','refusal'),row("No, it's dawn",'missing'),row('What is my name?','recalled','Sora',source_turn=1)],{'name.value':'Sora'})
add('external-focus', [row('I prefer quiet rooms','remembered'),row('what will the temperature be tomorrow',{'model':'intent','plain':'refusal'}),row("Actually, it's sunny",'missing'),row('What do I prefer?','recalled','quiet rooms',source_turn=1)],{'preferences.general.value':'quiet rooms'})
for text in ['My name is','I prefer','My favorite color is','Call me']:
    add('empty-'+text,[row('My name is Sol','remembered'),row(text,'refusal'),row("Actually, it's Rain",'missing'),row('What is my name?','recalled','Sol',source_turn=1)],{'name.value':'Sol'})
add('social-focus',[row('My favorite drink is Café','remembered'),row('Hello','social'),row("Actually, it's Oolong",'missing'),row('What is my favorite drink?','recalled','Café',source_turn=1)])
add('missing-reference',[row('My name is Yara','remembered'),row('What is my favorite flower?','missing'),row("Actually, it's iris",'missing')],{'name.value':'Yara','preferences':{},'last_slot':None})
add('last-utterance-focus',[row('My name is Yara','remembered'),row('What did I just say?','recalled','My name is Yara',source_turn=1),row("Actually, it's Mara",'missing')],{'name.value':'Yara'})
add('explicit-after-invalidation',[row('My name is Yara','remembered'),row('Thanks','social'),row('Actually, my name is Nia','remembered'),row('What is my name?','recalled','Nia',source_turn=3),row("No, it's Noor",'remembered','Noor')],{'last_correction.previous.value':'Nia','last_correction.previous.stated_on_turn':3,'name.value':'Noor','name.stated_on_turn':5})
add('learned-reads',[row('My name is Anouk','remembered'),row('by what name do you refer to me',{'model':'recalled','plain':'refusal'})],{'name.value':'Anouk'})
add('external-services',[row('set a timer for ten minutes',{'model':'intent','plain':'refusal'}),row('remind me to call my mother saturday morning',{'model':'intent','plain':'refusal'})],{'name':None,'preferences':{}})
add('restart-correction',[row('My favorite İÇECEK is Café con leche','remembered')],{'preferences.i̇çecek.value':'Café con leche'},more=[dict(rows=[row('What about that?','recalled','Café con leche',source_turn=1),row("No, it's Oolong",'remembered')],state={'preferences.i̇çecek.value':'Oolong','last_correction.previous.stated_on_turn':1}),dict(rows=[row('Thanks','social')],state={'last_slot':None}),dict(rows=[row("Actually, it's tea",'missing'),row('/reset','reset'),row('What is my name?','missing'),row('What did I just say?','recalled','What is my name?',source_turn=1)],state={'turn':2,'preferences':{},'name':None,'last_correction':None})])
add('legacy-session',[row('What is my name?','recalled','Zoë',source_turn=2),row("Actually, it's Ana",'remembered')],{'name.value':'Ana','last_correction.previous.stated_on_turn':2},initial={'turn':4,'name':{'value':'Zoë','stated_on_turn':2},'preferences':{},'last_slot':'Name','last_user_utterance':'hello'})
(root/'contracts.json').write_text(json.dumps(sessions,ensure_ascii=False,indent=2)+'\n')
