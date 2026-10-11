#!/usr/bin/env python3
"""Existing snapshot example, selected from a multivalued P364 edge, not DEV gold."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--bundle',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
subject='Q1123891';property='P364';question='What language is spoken in Brotherhood of the Wolf?'
index=json.loads((a.bundle/'index.json').read_text()); expected=sorted(set(index['entities'][subject]['claims'][property]));assert len(expected)>1
claims=[];source=None
for path in sorted(a.cache.glob('*.json')):
 raw=json.loads(path.read_text());entity=raw.get('entities',{}).get(subject)
 if entity:
  claims=[dict(statement_id=c.get('id'),target=c.get('mainsnak',{}).get('datavalue',{}).get('value',{}).get('id')) for c in entity.get('claims',{}).get(property,[])]
  source=dict(path=str(path),bytes=path.stat().st_size,sha256=hashlib.sha256(path.read_bytes()).hexdigest());break
assert set(expected)<={c['target'] for c in claims},claims
cmd=[a.binary,'talk','--facts-dir',str(a.bundle)]
reply=subprocess.run(cmd+['--json'],input='List answers to: '+question+'\n',text=True,capture_output=True,check=True,timeout=90).stdout
result=json.loads(reply);assert result['answers']==expected and result['entity']==subject and result['relation']==property,result
plain=subprocess.run(cmd,input='List answers to: '+question+'\n',text=True,capture_output=True,check=True,timeout=90).stdout
(a.output/'reply.json').write_text(reply);(a.output/'reply.txt').write_text(plain)
report=dict(question=question,subject=subject,property=property,expected_ids_from_edges=expected,published_claims=claims,raw_source=source,source='Wikidata cached snapshot retrieved 2026-10-10',license='CC0-1.0',new_download_bytes=0)
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
