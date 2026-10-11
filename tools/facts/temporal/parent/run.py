"""Run unchanged parent semantics through the submitted extractor, library and CLI."""
from pathlib import Path
import hashlib,json,subprocess
base=Path(__file__).parent
work=base.parents[1]
out=base/'candidate';out.mkdir()
frozen=work/'temporal-run/frozen-candidate'
extract=work/'temporal-run/terminal-originals/source/tools/facts/temporal/extract.py'
cases=json.loads((base/'cases.json').read_text())
assert hashlib.sha256((base/'cases.json').read_bytes()).hexdigest()=='b4b37ca4340afd79227d7102535430592a9961a152d4e8f0f1a69d3c829481f5'
rows=[]
for c in cases:
 d=out/c['name'];d.mkdir();src=base/'inputs'/c['name']
 raw=(src/'entity.json').read_bytes();index=(src/'index.json').read_bytes()
 name=hashlib.sha256(raw).hexdigest()[:16]+'.json'
 (d/name).write_bytes(raw);(d/'index.json').write_bytes(index)
 manifest=json.loads((src/'manifest.json').read_text());manifest['files'][0]['file']=name
 (d/'manifest.json').write_text(json.dumps(manifest))
 (d/'index-manifest.json').write_text(json.dumps({'index.json':{'bytes':len(index),'sha256':hashlib.sha256(index).hexdigest()}}))
 p=subprocess.run(['python',str(extract),'--source',str(d),'--index',str(d/'index.json'),'--index-manifest',str(d/'index-manifest.json'),'--output',str(d/'temporal.json')],capture_output=True,text=True,timeout=30)
 (d/'extract.stdout').write_text(p.stdout);(d/'extract.stderr').write_text(p.stderr)
 assert p.returncode==0,(c['name'],p.stderr)
 p=subprocess.run([str(frozen/'binary/temporal_proof'),str(d/'index.json'),str(d/'temporal.json')],input=json.dumps([c['date'],c['question']])+'\n',text=True,capture_output=True,timeout=30)
 (d/'library.stdout').write_text(p.stdout);(d/'library.stderr').write_text(p.stderr)
 assert p.returncode==0,(c['name'],p.stderr)
 value=json.loads(p.stdout);cli_equal=None
 if '\0' not in c['question']:
  cli=subprocess.run([str(frozen/'binary/glassmind-facts'),'verify-at','--index',str(d/'index.json'),'--temporal',str(d/'temporal.json'),'--date',c['date'],c['question']],capture_output=True,text=True,timeout=30)
  (d/'cli.stdout').write_text(cli.stdout);(d/'cli.stderr').write_text(cli.stderr)
  cli_equal=cli.returncode==0 and json.loads(cli.stdout)==value
 witness=value.get('witness',{});edge=witness.get('edge',{})
 edge_ok=value['status']!='supported' or [edge.get(k) for k in ['subject','property','object']]==c['expected_witness_edge']
 row=dict(name=c['name'],expected=c['expected_status'],actual=value['status'],status_ok=value['status']==c['expected_status'],edge_ok=edge_ok,cli_equal=cli_equal)
 rows.append(row);print(json.dumps(row),flush=True)
 (out/'report.json').write_text(json.dumps(rows,indent=2)+'\n')
summary={'total':len(rows),'status_pass':sum(r['status_ok'] for r in rows),'edge_pass':sum(r['edge_ok'] for r in rows),'cli_compared':sum(r['cli_equal'] is not None for r in rows),'cli_equal':sum(r['cli_equal'] is True for r in rows)}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
