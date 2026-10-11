"""Native graph invariants with fixed expected outcomes from the frozen graph."""
import argparse,json,subprocess,copy
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
r=Path(__file__).parent;original=json.loads((r/'graph.json').read_text());results=[]
def run(name,graph,q,ids,status='matched'):
 file=a.output/(name+'.json');file.write_text(json.dumps(graph)+'\n')
 got=json.loads(subprocess.run([a.binary,'match','--index',file,q],capture_output=True,check=True).stdout)
 assert got['status']==status,got
 if status=='matched':
  assert got['total_snapshot_matches']==len(ids) and got['returned']==min(10,len(ids)) and got['truncated']==(len(ids)>10)
  assert [m['subject'] for m in got['matches']]==ids[:10]
  for m in got['matches']:
   assert len(m['witnesses'])==len(got['constraints'])
   for w,c in zip(m['witnesses'],got['constraints']):
    assert w['subject']==m['subject'] and w['property']==c['property'] and w['object']==c['object']
    assert w['object'] in graph['entities'][w['subject']]['claims'][w['property']]
    assert w['source_url']==f"https://www.wikidata.org/wiki/{w['subject']}#{w['property']}"
 results.append({'name':name,'result':got});return got
q='Who was born in Q900 and works for Q901?'
g=copy.deepcopy(original)
for i in range(300,313):g['entities'][f'Q{i}']['claims'].pop('P108')
run('beyond-first-ten',g,q,['Q313'])
g['entities']=dict(reversed(list(g['entities'].items())))
for e in g['entities'].values():
 for p,values in e['claims'].items():e['claims'][p]=list(reversed(values))*2
run('reversed-duplicated',g,q,['Q313'])
run('empty-store',{'entities':{}},q,[])
c=json.loads((r/'separator-contract.json').read_text())
run('separator',c['graph'],c['question'],[],'refused')
for e in c['graph']['entities'].values():e['claims'].clear()
run('separator-no-claims',c['graph'],c['question'],[],'refused')
(a.output/'results.json').write_text(json.dumps(results,indent=2)+'\n');print({'native_invariants_passed':len(results)})
