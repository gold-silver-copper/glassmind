"""Search only permitted finite index for a strict conjunction, then verify CLI witnesses."""
import argparse,collections,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--index',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
data=a.index.read_bytes();graph=json.loads(data)['entities'];inverted=collections.defaultdict(set)
phrases={'P19':'was born in','P20':'died in','P27':'is a citizen of','P108':'works for'}
for s,e in graph.items():
 for p,objects in e['claims'].items():
  if p in phrases:
   for o in objects:inverted[p,o].add(s)
found=None
for c1 in sorted(inverted):
 for c2 in sorted(inverted):
  if c1>=c2 or c1[0]==c2[0]:continue
  shared=inverted[c1]&inverted[c2]
  if shared and len(shared)<min(len(inverted[c1]),len(inverted[c2])):
   found=c1,c2,shared;break
 if found:break
assert found
c1,c2,shared=found;q=f'Who {phrases[c1[0]]} {c1[1]} and {phrases[c2[0]]} {c2[1]}?'
r=subprocess.run([a.binary,'match','--index',a.index,q],capture_output=True,text=True,check=True,timeout=10);got=json.loads(r.stdout)
assert got['status']=='matched' and got['total_snapshot_matches']==len(shared)
assert [m['subject'] for m in got['matches']]==sorted(shared)[:10]
for m in got['matches']:
 assert {(w['property'],w['object']) for w in m['witnesses']}=={c1,c2}
 for w in m['witnesses']:
  assert w['subject']==m['subject'] and w['object'] in graph[w['subject']]['claims'][w['property']]
  assert w['source_url']==f"https://www.wikidata.org/wiki/{w['subject']}#{w['property']}"
report=dict(question=q,single_filter_sets=[sorted(inverted[c1]),sorted(inverted[c2])],intersection=sorted(shared),result=got,index_sha256=hashlib.sha256(data).hexdigest(),scope='finite validation-subject-seeded index capability evidence, not independent benchmark or open-world truth')
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
