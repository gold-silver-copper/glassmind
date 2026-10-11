"""Replay preregistered parent contracts; compare raw-source witnesses independently."""
from pathlib import Path
import hashlib,json,subprocess
base=Path(__file__).parent;work=base.parents[1];frozen=work/'timedmatch-run/frozen-candidate';out=base/'candidate';out.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=json.loads((frozen/'manifest.json').read_text())
for r in manifest['files']:
 p=frozen/r['path'];assert p.stat().st_size==r['bytes'] and sha(p)==r['sha256'],p
assert sha(base/'cases.json')==json.loads((base/'freeze.json').read_text())['sha256']
assert sha(base/'real-contracts.json')==json.loads((base/'oracle-summary.json').read_text())['real_sha256']
proof=base/'parent_timed_matching';cli=frozen/'bin/glassmind-facts';cases=json.loads((base/'cases.json').read_text());oracle={r['name']:r for r in json.loads((base/'authored-oracle.json').read_text())};records=[]
def native(index,temporal,queries):
 r=subprocess.run([str(proof),str(index),str(temporal)],input=''.join(json.dumps(q)+'\n' for q in queries),text=True,capture_output=True,timeout=60);assert r.returncode==0,r.stderr;rows=[json.loads(s) for s in r.stdout.splitlines()];assert len(rows)==len(queries);return rows
def verify(value,expect,date,index,temporal):
 assert value['status']=='matched' and value['date']==date
 assert value['total_recorded_interval_matches']==len(expect) and value['returned']==min(10,len(expect)) and value['truncated']==(len(expect)>10)
 assert [m['subject'] for m in value['matches']]==[m['subject'] for m in expect[:10]]
 assert value['scope']=='local_snapshot_recorded_bounded_intervals' and 'not global absence' in value['semantics']
 meta=json.loads(temporal.read_text());idxsha=sha(index)
 constraints=[(x['property'],x['object']) for x in value['constraints']]
 assert constraints==sorted(set(constraints))
 count=0
 for got,wanted in zip(value['matches'],expect[:10]):
  assert len(got['witnesses'])==len(wanted['witnesses'])==len(constraints)
  for g,e,cs in zip(got['witnesses'],wanted['witnesses'],value['constraints']):
   edge=g['edge'];assert all(edge[k]==e[k] for k in ['subject','property','object']);assert edge['subject_resolution']=='explicit_id' and edge['object_resolution']==cs['object_resolution']
   assert g['claim_id']==e['claim_id'];assert [g['start']['earliest'],g['start']['latest']]==e['start_range'];assert [g['end']['earliest'],g['end']['latest']]==e['end_range'];assert g['start']['latest']<date<g['end']['earliest'];assert g['index_sha256']==idxsha and g['policy']=='latest_start_day < requested_day < earliest_end_day'
   entries=[r for r in meta['records'] if r['statement'].get('id')==g['claim_id']];assert len(entries)==1;record=entries[0];s=record['statement'];assert g['rank']==s['rank']
   for key in ['raw_file','raw_sha256','source_url']:assert g[key]==record[key]
   for key,prop in [('start','P580'),('end','P582')]:assert g[key]['raw']==s['qualifiers'][prop][0]['datavalue']['value']
   assert edge['source_url']==f"https://www.wikidata.org/wiki/{e['subject']}#{e['property']}";count+=1
 return count
witnesses=0;cli_cases=0
for c in cases:
 d=base/'inputs'/c['name'];v=native(d/'index.json',d/'temporal.json',[[c['date'],c['question']]])[0];(out/(c['name']+'.json')).write_text(json.dumps(v,indent=2)+'\n');assert v['status']==c['expected_status'],(c['name'],v)
 if v['status']=='matched':witnesses+=verify(v,oracle[c['name']]['matches'],c['date'],d/'index.json',d/'temporal.json')
 if '\0' not in c['question']:
  r=subprocess.run([str(cli),'match-at','--index',str(d/'index.json'),'--temporal',str(d/'temporal.json'),'--date',c['date'],c['question']],capture_output=True,text=True,timeout=30);assert r.returncode==0,r.stderr;assert json.loads(r.stdout)==v,c['name'];cli_cases+=1
 records.append({'name':c['name'],'status':v['status']})
real=json.loads((base/'real-contracts.json').read_text());idx=work/'factmatch-source/index.json';temp=work/'timedmatch-source/temporal.json';results=native(idx,temp,[[r['date'],r['question']] for r in real]);(out/'real.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in results));realw=0
for r,v in zip(real,results):realw+=verify(v,r['expected_matches'],r['date'],idx,temp)
report={'manifest_entries_verified':len(manifest['files']),'authored_cases':len(records),'cli_library_equal':cli_cases,'nul_library_only':len(records)-cli_cases,'authored_full_witnesses':witnesses,'real_queries':len(real),'real_nonempty':sum(bool(r['expected_matches']) for r in real),'real_full_witnesses':realw,'cases':records,'proof_sha256':sha(proof),'cli_sha256':sha(cli),'scope':'Parent frozen authored and complete eligible cached distinct pairs; cache validation-subject seeded; no independent retrieval claim'};(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
