"""Public authored prospective contracts, created before runtime edits."""
import json
from pathlib import Path
social=['goodbye','hey there!','see you later','thanks so much!','nice to speak with you','how are things going','Hello']
contrasts=[
 ('Elin went to the garden.', 'story_fact','garden',False),
 ('Elin teleported to the garden.', 'refusal',None,True),
 ('hello Elin teleported to the garden', 'refusal',None,True),
 ('hello; Elin went to the garden.', 'refusal',None,True),
 ('hey there and Elin moved to the garden', 'refusal',None,True),
 ('goodbye and book a flight', 'refusal',None,True),
 ('thanks so much, Elin went to the garden', 'refusal',None,True),
 ('Elin did not move to the garden.', 'refusal',None,True),
 ('Elin said "goodbye".', 'refusal',None,True),
 ('"hello"', 'refusal',None,True),
 ('not goodbye', 'refusal',None,True),
 ('Did Elin teleport to the garden?', None,'kitchen',False),
 ('Where is Elin?', 'story_answer','kitchen',False),
 ('Elin is not in the kitchen.', 'story_fact',None,False),
 ('Actually, Elin is in the garden.', 'story_fact','garden',False),
]
rows=[dict(type='social',context=c,text=s) for s in social for c in ['none','active','invalid']]
rows += [dict(type='contrast',text=t,kind=k,answer=a,invalidated=i) for t,k,a,i in contrasts]
rows += [dict(type='priority',text=t,kind=k) for t,k in [('My name is Nia','remembered'),('My favorite color is green','remembered'),('Calculate 7 * 8','calculated'),('Call me','refusal'),('Book a flight','intent')]]
Path(__file__).with_name('contracts.json').write_text(json.dumps(rows,indent=2)+'\n')
