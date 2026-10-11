from pathlib import Path
import hashlib,json
w=Path('/home/gsc/.cache/glassmind/continuous-20261010');src=w/'factmatch-source';cache=Path('/home/gsc/.cache/glassmind/data/simplequestions/wikidata');out=Path(__file__).parent
manifest=json.loads((src/'index-source.json').read_text());examples=json.loads((out/'index-audit.json').read_text())['examples'];wanted={s for ex in examples for s in ex['matches']};found={};filehashes={}
for rec in manifest['wikidata_responses']:
 p=cache/rec['file'];data=p.read_bytes();assert len(data)==rec['bytes'];sha=hashlib.sha256(data).hexdigest();assert sha==rec['sha256'];j=json.loads(data)
 for q in wanted.intersection(j.get('entities',{})):
  found[q]=j['entities'][q];filehashes[q]={'file':rec['file'],'sha256':sha,'bytes':len(data)}
reports=[]
for ex in examples:
 witnesses=[]
 for s in ex['matches']:
  for c in ex['conditions']:
   prop,obj=c['property'],c['object'];matches=[]
   for claim in found[s].get('claims',{}).get(prop,[]):
    snak=claim.get('mainsnak',{});value=snak.get('datavalue',{}).get('value',{})
    if isinstance(value,dict) and value.get('id')==obj:
     matches.append({'claim_id':claim.get('id'),'rank':claim.get('rank'),'qualifier_properties':sorted(claim.get('qualifiers',{}))})
   assert matches,(s,prop,obj)
   witnesses.append({'subject':s,'property':prop,'object':obj,'source_url':f'https://www.wikidata.org/wiki/{s}#{prop}','cached_response':filehashes[s],'raw_claims':matches})
 reports.append(dict(ex,witnesses=witnesses))
result={'scope':'Exact cached directed edges only; historical/qualified claims and validation-subject seeded partial index, not current or global truth. No network or heldout data.', 'index_sha256':manifest['index_sha256'],'examples':reports}
(out/'raw-edge-audit.json').write_text(json.dumps(result,indent=2)+'\n');print('Confirmed',len(reports),'examples,',sum(len(r['witnesses']) for r in reports),'edge witnesses')
