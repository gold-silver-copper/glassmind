"""Select first supported-property edge lexically from index, check raw statements offline.
Selection uses only index graph, never DEV annotations or candidate outputs.
"""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--index',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
entities=json.loads(a.index.read_text())['entities']
forms={'P19':'Was {} born in {}?','P20':'Did {} die in {}?','P27':'Is {} a citizen of {}?','P108':'Does {} work for {}?'}
s,p,o=next((s,p,o) for s,e in sorted(entities.items()) for p,objects in sorted(e['claims'].items()) if p in forms for o in sorted(set(objects)))
question=forms[p].format(s,o);statement=None
for path in sorted(a.cache.glob('*.json')):
    raw=json.loads(path.read_text());entity=raw.get('entities',{}).get(s)
    if not entity: continue
    matches=[c for c in entity.get('claims',{}).get(p,[]) if c.get('mainsnak',{}).get('datavalue',{}).get('value',{}).get('id')==o]
    if not matches: continue
    statement=dict(subject=s,property=p,object=o,published_statements=[{'id':c['id'],'rank':c.get('rank'),'qualifiers':c.get('qualifiers',{})} for c in matches],cache_file=str(path),cache_bytes=path.stat().st_size,cache_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),lastrevid=entity.get('lastrevid'))
    break
assert statement is not None,(s,p,o)
a.output.mkdir(parents=True,exist_ok=True)
raw=subprocess.run([a.binary,'verify','--index',a.index,question],capture_output=True,check=True,timeout=10).stdout
(a.output/'reply.json').write_bytes(raw);got=json.loads(raw);assert got['status']=='supported'
w=got['witness'];assert [w['subject'],w['property'],w['object']]==[s,p,o]
report=dict(question=question,selection='first lexical subject/property/object over four supported properties; QID endpoints',claim=statement,scope='500 validation-subject-seeded entities; finite snapshot only',source='Wikidata existing cached snapshot retrieved 2026-10-10; CC0-1.0',new_download_bytes=0)
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
