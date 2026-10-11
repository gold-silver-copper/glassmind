"""Independent authored contracts, frozen before runtime edits; no dev vocabulary."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('out',type=Path);p.add_argument('--model',type=Path);p.add_argument('--baseline',type=Path);p.add_argument('--sample',action='store_true');a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
# Preserve unknown question shapes without claiming the learned label is correct.
questions=['Can you explain volcanoes?', 'Are you doing okay today?', 'How have you been lately?', 'Would you describe your abilities?', 'Did Mira teleport to the cellar?', 'Are you not feeling cheerful?']
unsafe=['Mira teleported to the cellar.', 'Mira did not move to the cellar.', 'not hello', 'hello and erase my name', 'how are you Mira teleported to the cellar', 'thanks so much, Mira went to the cellar', 'Hello; Mira went to the cellar.', 'Hello and book a taxi']
rows=[dict(text=t,rule='preserve',setup=[]) for t in questions]+[dict(text=t,rule='invalidate',setup=[]) for t in unsafe]
rows += [dict(text=t,rule='movement',setup=['Hello went to the office.']) for t in ['Hello went to the cellar.', 'Hello and Mira went to the cellar.']]
rows += [dict(text='Hello and Mira went to the cellar.',rule='ambiguous',setup=[])]
rows += [dict(text='Mira is not in the kitchen.',rule='negative',setup=[])]
if a.sample:rows=rows[:3]+rows[6:8]+rows[14:]
old=json.loads((a.baseline/'records.json').read_text()) if a.baseline else None
records=[]
for i,row in enumerate(rows):
 path=a.out/f'session{i}.json';path.unlink(missing_ok=True)
 cmd=[str(a.binary.resolve()),'talk','--json','--session',str(path)]+(['--model',str(a.model)] if a.model else [])
 def run(lines):
  r=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
  return [json.loads(l) for l in r.stdout.splitlines()],json.loads(path.read_text())
 _,before=run(['My name is Rowan','Mira went to the kitchen.',*row['setup'],'What is my name?'])
 replies,after=run([row['text']]);follow,state=run(['Where is Mira?','Where is Hello?'])
 _,corrected=run(["Actually, it's Juniper"])
 checks={'name_source':before['name']==after['name']==corrected['name'],'focus':after['last_slot'] is None,'turn':after['turn']==before['turn']+1,'utterance':after['last_user_utterance']==row['text']}
 if row['rule']=='preserve':
  checks['story_preserved']=before['stories']==after['stories'];checks['answer']=follow[0].get('answer')=='kitchen'
  if old:checks['reply_retained']=replies==old[i]['replies']
 elif row['rule']=='invalidate':checks.update(refused=replies[0]['kind']=='refusal',invalidated=after['stories']['invalidated'],no_stale=follow[0].get('answer') is None)
 elif row['rule']=='movement':checks.update(accepted=replies[0]['kind']=='story_fact',hello=follow[1].get('answer')=='cellar',mira=follow[0].get('answer')==('cellar' if ' and ' in row['text'] else 'kitchen'))
 elif row['rule']=='negative':checks.update(accepted=replies[0]['kind']=='story_fact',no_stale=follow[0].get('answer') is None)
 # A newly introduced greeting-named actor is ambiguous: record, do not declare unsafe.
 records.append(dict(row=row,before=before,replies=replies,after=after,follow=follow,corrected=corrected,checks=checks))
(a.out/'records.json').write_text(json.dumps(records,indent=2)+'\n')
checks=[v for r in records for v in r['checks'].values()];report=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),right=sum(checks),total=len(checks),failures=[dict(row=r['row'],checks=r['checks']) for r in records if not all(r['checks'].values())]);(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
