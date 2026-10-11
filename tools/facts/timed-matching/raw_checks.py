"""Retain each independently matched original claim identity and canonical hash."""
from freeze import *
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');O=W/'timedmatch-run/evidence';raw={}
a=json.loads((W/'timedmatch-source/temporal.json').read_bytes());out=[]
for r in a['records']:
    file=r['raw_file']
    if file not in raw:
        b=(W/'temporal-source'/file).read_bytes();assert digest(b)==r['raw_sha256'];raw[file]=json.loads(b)
    s=next(s for s in raw[file]['entities'][r['subject']]['claims'][r['property']] if s['id']==r['statement']['id'])
    assert s==r['statement']
    out.append(dict(subject=r['subject'],property=r['property'],object=r['object'],claim_id=s['id'],raw_file=file,raw_sha256=r['raw_sha256'],canonical_statement_sha256=digest(encode(s)),exact_original_statement=True,ranges=bounds(s)))
(O/'original-statement-checks.json').write_bytes(encode(out)+b'\n')
