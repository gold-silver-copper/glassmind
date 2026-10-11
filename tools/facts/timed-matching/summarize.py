from pathlib import Path
import json,hashlib,subprocess
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');R=W/'timedmatch-run';E=R/'evidence'
q=json.loads((E/'real/queries.json').read_text());rs=[json.loads(s) for s in (E/'real/full.jsonl').read_text().splitlines()]
distinct=[(x,r) for x,r in zip(q,rs) if len(r['constraints'])>1];positive=[(x,r) for x,r in distinct if r['returned']]
summary=dict(authored_frozen_cases=51,full_cache_queries=len(rs),distinct_constraint_queries=len(distinct),distinct_constraint_nonzero_queries=len(positive),distinct_positive_pairs=len({json.dumps(r['constraints'],sort_keys=True) for _,r in positive}),distinct_positive_subjects=sorted({m['subject'] for _,r in positive for m in r['matches']}),recorded_matches_nonzero=sum(r['returned']>0 for r in rs),witnesses=sum(len(m['witnesses']) for r in rs for m in r['matches']))
(E/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
p=subprocess.run([str(R/'baseline/glassmind-facts'),'match-at','--help'],capture_output=True)
(E/'baseline-new-command.json').write_text(json.dumps(dict(exit=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode())))
assert p.returncode!=0
# All new data are public authored development fixtures or derivatives of existing assets.
files=[W/'factmatch-source'/s for s in ['index.json','relation.json','valid.tsv','source.json','index-source.json','LICENSE.txt']]+[W/'timedmatch-source/temporal.json',Path('tools/facts/timed-matching/contracts.json')]
manifest=[]
for p in files:
    b=p.read_bytes();manifest.append(dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
(E/'data-identities.json').write_text(json.dumps(manifest,indent=2)+'\n')
