from pathlib import Path
import json,subprocess,hashlib
root=Path(__file__).resolve().parent; w=root.parents[2]; fixtures=w/'factverify/tools/facts/verification'; binary=root/'frozen/glassmind-facts'; outcomes=[]
supp=json.loads((fixtures/'supplemental.json').read_text());(root/'supplemental-store.json').write_text(json.dumps(supp['graph']))
for group,graph,cases in [('original67',fixtures/'graph.json',json.loads((fixtures/'cases.json').read_text())),('supplemental5',root/'supplemental-store.json',supp['cases'])]:
 for i,c in enumerate(cases):
  if '\x00' in c['question']:continue
  r=subprocess.run([str(binary),'verify','--index',str(graph),c['question']],capture_output=True,text=True,timeout=10);assert r.returncode==0
  actual=json.loads(r.stdout);passed=actual['status']==c['status']
  if c['status']=='supported':passed &= [actual.get('witness',{}).get(k) for k in ('subject','property','object')]==c['edge']
  outcomes.append(dict(group=group,index=i,original=c,actual=actual,pass_=passed))
for c in json.loads((root/'contrasts.json').read_text()):
 graph=root/(c['name']+'.json');graph.write_text(json.dumps(c['graph']))
 r=subprocess.run([str(binary),'verify','--index',str(graph),c['text']],capture_output=True,text=True,timeout=10);assert r.returncode==0
 actual=json.loads(r.stdout);outcomes.append(dict(group='parent_post_discovery',name=c['name'],actual=actual,pass_=actual['status']==c['expected']))
(root/'outcomes.json').write_text(json.dumps(outcomes,indent=2)+'\n');print(json.dumps({'total':len(outcomes),'passed':sum(x['pass_'] for x in outcomes),'failures':[x for x in outcomes if not x['pass_']]},indent=2))
