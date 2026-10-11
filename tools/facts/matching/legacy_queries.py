"""Read only public authored verification contracts and permitted dev questions."""
import argparse,subprocess,json,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--baseline',type=Path,required=True);p.add_argument('--candidate',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
r=Path('tools/facts/verification');data=Path('/home/gsc/.cache/glassmind/continuous-20261010/factmatch-source');results=[];old_disagreements=[]
for filename in ['cases.json','cases-alias-v2.json']:
 cases=json.loads((r/filename).read_text())
 for i,c in enumerate(cases):
  if '\0' in c['question']:continue
  args=['verify','--index',r/'graph.json',c['question']]
  before=subprocess.run([a.baseline,*args],capture_output=True,check=True).stdout;after=subprocess.run([a.candidate,*args],capture_output=True,check=True).stdout
  assert before==after,(filename,i)
  got=json.loads(after)
  if filename=='cases-alias-v2.json':
   assert got['status']==c['status'],(i,c,got)
   if got['status']=='supported':assert [got['witness'][k] for k in ['subject','property','object']]==c['edge']
  elif got['status']!=c['status']:old_disagreements.append({'case':i,'question':c['question'],'old_expected':c['status'],'baseline_and_candidate':got})
  results.append({'source':filename,'case':i,'byte_identical':True,'result':got})
# Exact standalone ask bytes for first 20 dev questions; models see questions only.
for i,row in enumerate((data/'valid.tsv').read_text().splitlines()[:20]):
 q=row.split('\t',3)[3];args=['ask','--model',data/'relation.json','--index',data/'index.json',q]
 before=subprocess.run([a.baseline,*args],capture_output=True,check=True).stdout;after=subprocess.run([a.candidate,*args],capture_output=True,check=True).stdout
 assert before==after,i
 results.append({'source':'valid first20 ask','case':i,'byte_identical':True,'result':json.loads(after)})
(a.output/'results.json').write_text(json.dumps(results,indent=2)+'\n');(a.output/'old-contract-disagreements.json').write_text(json.dumps(old_disagreements,indent=2)+'\n')
summary={'byte_identical_cases':len(results),'old_contract_disagreements':len(old_disagreements),'current_verification_contracts':'all passed'};(a.output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
