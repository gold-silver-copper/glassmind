"""Query actual submitted private pair/gradient functions through a parent wrapper."""
from pathlib import Path
import hashlib,json,math,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parent.parent
binary=HERE/'probe-native';shutil.copy2(ROOT/'equivbinding-run/target/release/equivalent-binding-helper',binary)
fixtures=json.loads((HERE/'cases.json').read_text());requests=[];expected=[]
def arg(i):
    if i<16:return dict(kind='input',index=i)
    if i<36:return dict(kind='constant',index=i-16)
    return dict(kind='step',index=i-36)
def step(s):return dict(op=s['op'],left=arg(s['left']),right=arg(s['right']))
for c in fixtures['valid']:
    requests.append(dict(kind='pairs',prefix=dict(steps=[step(s) for s in c['prefix']]),step=step(c),inputs=c['input_count']));expected.append(c['expected_pairs'])
for c in fixtures['invalid']:
    request=dict(kind='pairs',prefix=dict(steps=[]),step=step(c),inputs=c['input_count'])
    if c['name']=='invalid_constant':request['step']['left']=dict(kind='constant',index=20)
    requests.append(request);expected.append(None)
def lse(v):
    m=max(v);return m+math.log(sum(math.exp(x-m) for x in v))
gradients=[]
for scores in [[-.91,1.47,2.36,-3.2],[0.,0.,0.,0.],[-900.,0.,20.,-5.]]:
    actions=[0,2,17,36]
    for selected in [[0],[17],[0,36],[2,17,36],actions]:
        logs=[s-lse(scores) for s in scores]
        requests.append(dict(kind='gradient',logps=logs,actions=actions,targets=selected))
        objective=lambda v:lse([v[i] for i,a in enumerate(actions) if a in selected])-lse(v)
        value=objective(scores);numerical=[]
        for i in range(4):
            hi=list(scores);lo=list(scores);hi[i]+=1e-5;lo[i]-=1e-5
            numerical.append((objective(hi)-objective(lo))/2e-5)
        gradients.append(dict(scores=scores,actions=actions,targets=selected,log_mass=value,numerical_gradient=numerical))
r=subprocess.run([str(binary)],input=''.join(json.dumps(v)+'\n' for v in requests),text=True,capture_output=True,check=True,timeout=60)
output=[json.loads(s) for s in r.stdout.splitlines()];assert len(output)==len(requests)
for want,actual in zip(expected,output[:len(expected)],strict=True):
    if want is None:assert 'Err' in actual
    else:assert actual=={'Ok':want},(want,actual)
max_error=0.
for record,actual in zip(gradients,output[len(expected):],strict=True):
    mass,gradient=actual['Ok'];assert abs(mass-record['log_mass'])<1e-10
    error=max(abs(a-b) for a,b in zip(gradient,record['numerical_gradient'],strict=True));max_error=max(max_error,error);assert error<1e-6
    record['native_gradient']=gradient
report=dict(formal_pairs=13,invalid_references=3,gradient_cases=len(gradients),gradient_coordinates=4*len(gradients),max_finite_difference_error=max_error,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),scope='Parent wrapper appended to archived original module; no fitting or runtime edits')
(HERE/'candidate/probe-requests.jsonl').write_text(''.join(json.dumps(v)+'\n' for v in requests));(HERE/'candidate/probe-outputs.jsonl').write_text(r.stdout)
(HERE/'candidate/gradient-checks.json').write_text(json.dumps(gradients,indent=2)+'\n');(HERE/'candidate/probe-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
