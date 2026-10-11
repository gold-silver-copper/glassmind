"""Finite cache coverage, independently checked against original raw bytes."""
import itertools, subprocess, sys
from freeze import *
W=Path('/home/gsc/.cache/glassmind/continuous-20261010'); R=W/'timedmatch-run'; O=R/'evidence/real';O.mkdir(exist_ok=True)
ib=(W/'factmatch-source/index.json').read_bytes();tb=(W/'timedmatch-source/temporal.json').read_bytes()
assert digest(tb)=='55097953db9b0f041ec4a7c2b5a7513586b1db9f572856cb84aa8610f484ee3d'
a=json.loads(tb); assert a['index_sha256']==digest(ib)
raw={}; identities=[]
for f in json.loads((W/'temporal-source/manifest.json').read_text())['files']:
    b=(W/'temporal-source'/f['file']).read_bytes(); assert len(b)==f['bytes'] and digest(b)==f['sha256'];raw[f['file']]=json.loads(b);identities.append(f)
original=[]
for r in a['records']:
    assert next(f for f in identities if f['file']==r['raw_file'])['sha256']==r['raw_sha256']
    claims=raw[r['raw_file']]['entities'][r['subject']]['claims'][r['property']]
    matches=[s for s in claims if s['id']==r['statement']['id']]; assert len(matches)==1 and matches[0]==r['statement']
    assert matches[0]['mainsnak']['datavalue']['value']['id']==r['object']
    original.append(dict(r,statement=matches[0]))
eligible=[r for r in original if bounds(r['statement'])]
by_subject={}
for r in eligible: by_subject.setdefault(r['subject'],set()).add((r['property'],r['object']))
pairs=set()
for cs in by_subject.values():
    pairs.update(itertools.combinations(sorted(cs),2));pairs.update((c,c) for c in cs)
queries=[];expectations=[]
for pair in sorted(pairs):
    dates={'2000-01-01'}
    for r in eligible:
        if (r['property'],r['object']) not in pair:continue
        b=bounds(r['statement'])
        for day in (b[0]['latest'],b[1]['earliest']):
            d=dt.date.fromisoformat(day)
            for delta in (-1,0,1):
                try:dates.add(str(d+dt.timedelta(days=delta)))
                except OverflowError: pass
    for date in sorted(dates):
        queries.append([date,question(pair)]);expectations.append(expected(ib,original,date,pair))
(O/'queries.json').write_bytes(encode(queries));(O/'expected.json').write_bytes(encode(expectations))
(O/'raw-checks.json').write_bytes(encode(dict(records=len(original),eligible=len(eligible),raw_files=identities,distinct_pairs=sum(p[0]!=p[1] for p in pairs),duplicate_pairs=sum(p[0]==p[1] for p in pairs),queries=len(queries))))
# Query freeze precedes any candidate invocation.
(O/'freeze.sha256').write_text('\n'.join(f'{digest((O/p).read_bytes())}  {p}' for p in ['queries.json','expected.json','raw-checks.json'])+'\n')
binary=R/'target/release/examples/timed_matching_proof'
for phase,n in [('sample',8),('full',len(queries))]:
    inp=b''.join(encode(q)+b'\n' for q in queries[:n])
    run=subprocess.run([str(binary),str(W/'factmatch-source/index.json'),str(W/'timedmatch-source/temporal.json')],input=inp,capture_output=True,check=True)
    (O/(phase+'.jsonl')).write_bytes(run.stdout)
    replies=[json.loads(l) for l in run.stdout.splitlines()];assert len(replies)==n
    for i,(reply,exp) in enumerate(zip(replies,expectations[:n])):
        assert reply['status']=='matched',(i,reply)
        for k,v in exp.items(): assert reply[k]==v,(phase,i,queries[i],k,reply[k],v)
    (O/(phase+'-report.json')).write_bytes(encode(dict(queries=n,nonzero=sum(x['returned']>0 for x in replies),witnesses=sum(sum(len(m['witnesses']) for m in x['matches']) for x in replies),exact=True)))
print((O/'raw-checks.json').read_text());print((O/'full-report.json').read_text())
