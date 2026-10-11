from pathlib import Path
from fractions import Fraction as F
import ast,hashlib,json,subprocess
HERE=Path(__file__).resolve().parent
FROZEN=HERE.parents[1]/'unitcompare-run/frozen-candidate'
OUT=HERE/'candidate';OUT.mkdir(exist_ok=False)
for name,sha in json.loads((HERE/'freeze.json').read_text()).items():
    assert hashlib.sha256((HERE/name).read_bytes()).hexdigest()==sha,name
cases=json.loads((HERE/'cases.json').read_text())
node=next(n for n in ast.parse((HERE/'freeze.py').read_text()).body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='units' for t in n.targets))
scope={'F':F};exec(compile(ast.Module(body=[node],type_ignores=[]),'<frozen oracle>','exec'),scope);units=scope['units']
def run(binary,items,args=()):
    p=subprocess.run([str(FROZEN/binary),*args],input=''.join(json.dumps(x)+'\n' for x in items),text=True,capture_output=True,timeout=30,check=True)
    (OUT/(binary+'.jsonl')).write_text(p.stdout)
    rows=[json.loads(s) for s in p.stdout.splitlines()];assert len(rows)==len(items);return rows
rows=run('final-native',[c['question'] for c in cases],['--compare-jsonl'])
positive=[];typed=[]
pred={'greater':'greater_than','less':'less_than','equal':'equal_to','at_least':'at_least','at_most':'at_most'}
for c,r in zip(cases,rows):
    if c['status']=='refused':assert r['status']=='refusal',(c,r);continue
    assert r['status']=='comparison',(c,r)
    v=r['result'];assert v['dimension']==c['dimension'] and v['ordering']==c['ordering'],(c,v)
    for side in ['left','right']:
        assert F(v[side]['value'])==F(c[side]['value']),(c,v)
    outcome=v['outcome']
    if c['truth'] is not None:
        assert outcome==dict(kind='predicate',predicate=pred[c['operation']],truth=c['truth']),(c,v)
        operation=dict(kind='predicate',operation=pred[c['operation']])
    else:
        adjective=c['question'].split(',')[0].split()[-1]
        assert outcome==dict(kind='selection',selection=adjective,selected=c['choice']),(c,v)
        operation=dict(kind='selection',operation=adjective)
    _,scale,offset=units[c['left']['unit']]
    witness=v['witness'];assert witness['unit']==v['left']['unit']
    assert F(witness['left'])*scale+offset==F(c['si_left']),(c,v)
    assert F(witness['right'])*scale+offset==F(c['si_right']),(c,v)
    # Reuse normalized unit spelling only, never the parsed numeric result or operation.
    typed.append(dict(left=dict(value=c['left']['value'],unit=v['left']['unit']),right=dict(value=c['right']['value'],unit=v['right']['unit']),operation=operation))
    positive.append(r)
direct=run('compare-typed',typed)
assert direct==positive,(direct[:1],positive[:1])
report=dict(total=len(cases),compared=len(positive),refused=len(cases)-len(positive),exact_si_witnesses=len(positive),direct_api_equal=len(direct),binary_sha256=hashlib.sha256((FROZEN/'final-native').read_bytes()).hexdigest(),cases_sha256=hashlib.sha256((HERE/'cases.json').read_bytes()).hexdigest())
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
