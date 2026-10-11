"""Native CLI evidence against preimplementation expectations; never changes labels."""
import argparse,json,hashlib,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
r=Path(__file__).parent;a.output.mkdir(parents=True,exist_ok=True)
graph=json.loads((r/'graph.json').read_text())['entities'];cases=json.loads((r/'contracts.json').read_text())
results=[];failures=[]
for i,c in enumerate(cases):
 if '\0' in c['question']:continue # argv cannot carry NUL; covered in Rust.
 out=subprocess.run([a.binary,'match','--index',r/'graph.json',c['question']],capture_output=True,text=True,timeout=10)
 try:
  assert out.returncode==0,out.stderr
  got=json.loads(out.stdout);assert got['status']==c['status'],got
  assert got['index_sha256']==hashlib.sha256((r/'graph.json').read_bytes()).hexdigest()
  if c['status']=='matched':
   ids=c['ids'];assert ids==sorted(set(ids))
   assert got['total_snapshot_matches']==len(ids) and got['returned']==min(10,len(ids)) and got['truncated']==(len(ids)>10)
   assert [m['subject'] for m in got['matches']]==ids[:10]
   expected=sorted(c['conditions']);assert [[x['property'],x['object']] for x in got['constraints']]==expected
   assert got['scope']=='local_snapshot' and 'temporal' in got['semantics']
   for m in got['matches']:
    s=m['subject'];assert [[w['property'],w['object']] for w in m['witnesses']]==expected
    for w in m['witnesses']:
     assert w['subject']==s and w['object'] in graph[s]['claims'][w['property']]
     assert w['source_url']==f"https://www.wikidata.org/wiki/{s}#{w['property']}"
  results.append({'case':i,'result':got})
 except Exception as e: failures.append({'case':i,'error':str(e),'stdout':out.stdout,'stderr':out.stderr})
(a.output/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
(a.output/'failures.json').write_text(json.dumps(failures,ensure_ascii=False,indent=2)+'\n')
summary={'passed':len(results),'failed':len(failures),'nul_library_only':1};(a.output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
assert not failures, failures
