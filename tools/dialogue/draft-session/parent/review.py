"""Replay frozen parent contracts through the submitted native API driver."""
from pathlib import Path
import copy, hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
BINARY = ROOT / 'draftsession-run/frozen-candidate/binaries/draft-session'
OUT = HERE / 'candidate'
OUT.mkdir(exist_ok=True)

class Driver:
    def __init__(self, bundle='valid'):
        self.p = subprocess.Popen([str(BINARY), str(HERE/'bundles'/bundle)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    def call(self, op, session='parent', **args):
        request = dict(session=session, command=dict(op=op, **args))
        self.p.stdin.write(json.dumps(request, ensure_ascii=False)+'\n'); self.p.stdin.flush()
        result = json.loads(self.p.stdout.readline())
        with (OUT/'events.jsonl').open('a') as f: f.write(json.dumps(dict(request=request,response=result),ensure_ascii=False)+'\n')
        return result
    def ok(self, op, **args):
        r=self.call(op, **args); assert r['ok'], r; return r['result']
    def close(self):
        self.p.stdin.close(); assert self.p.wait(timeout=20)==0, self.p.stderr.read()

assert not (OUT/'events.jsonl').exists(), 'Preserve old runs; choose a new output directory'
report = {'binary_sha256':hashlib.sha256(BINARY.read_bytes()).hexdigest()}
invalid = json.loads((HERE/'loader-contracts.json').read_text())['invalid_bundles'] + ['model_version']
report['invalid_bundles'] = []
for name in invalid:
    p=subprocess.run([str(BINARY),str(HERE/'bundles'/name)],input='',text=True,capture_output=True,timeout=30)
    assert p.returncode!=0,(name,p.stdout)
    report['invalid_bundles'].append(dict(name=name,error=p.stderr.strip()))

d=Driver()
empty=d.ok('show'); assert empty['provisional'] and empty['draft'] is None and empty['total_turns']==0
history=d.ok('save')['history']; assert set(history)=={'version','bundle_identity','turns'}
assert history['turns']==[] and d.ok('restore',history=history)==empty
streams=json.loads((HERE/'streams.json').read_text())
expected=json.loads((HERE/'baseline-predictions.json').read_text())
report['authored_user_predictions']=0;report['restart_points']=0
for i,s in enumerate(streams):
    name=s['name']; preds=[]
    for n,t in enumerate(s['turns'],1):
        snap=d.ok(t['speaker'].lower(),session=name,utterance=t['utterance'])
        assert snap['provisional'] and snap['total_turns']==n
        assert d.ok('show',session=name)==snap
        saved=d.ok('save',session=name)['history'];assert saved['turns']==s['turns'][:n]
        assert all(set(t)=={'speaker','utterance'} for t in saved['turns'])
        if t['speaker']=='USER':preds.append(snap['draft'])
        if n in s['restart_after']:
            resumed=Driver(); assert resumed.ok('restore',history=saved)==snap
            for following in s['turns'][n:]:resumed.ok(following['speaker'].lower(),utterance=following['utterance'])
            final_saved=resumed.ok('save')['history']; assert final_saved['turns']==s['turns']
            resumed.close();report['restart_points']+=1
    assert preds==expected[i],(name,'prediction mismatch')
    report['authored_user_predictions']+=len(preds)
    assert d.ok('reset',session=name)==empty

# All malformed histories must leave an already populated session byte-identical.
d.ok('user',utterance='Find attractions in Kyoto.')
before=d.ok('show'); saved=d.ok('save')['history']
bad={}
bad['unknown_top_field']=dict(saved,frames=[])
x=copy.deepcopy(saved);x['turns'][0]['frames']=[];bad['annotation_bearing_turn']=x
x=copy.deepcopy(saved);x['turns'][0]['speaker']='ASSISTANT';bad['unknown_speaker']=x
bad['bad_version']=dict(saved,version=999)
bad['wrong_identity']=dict(saved,bundle_identity='0'*64)
for name,text in [('empty_utterance',''),('nul_utterance','a\0b'),('multibyte_over_4096','é'*2049)]:
    bad[name]=dict(saved,turns=[dict(speaker='USER',utterance=text)])
bad['129_turns']=dict(saved,turns=[dict(speaker='SYSTEM',utterance='x')]*129)
bad['65537_total_bytes']=dict(saved,turns=[dict(speaker='SYSTEM',utterance='x'*4096)]*16+[dict(speaker='SYSTEM',utterance='x')])
for name,h in bad.items():
    assert not d.call('restore',history=h)['ok'],name
    assert d.ok('save')['history']==saved and d.ok('show')==before,name
report['invalid_history_atomic']=list(bad)
for name in ['model_rebound_same_semantics','schema_rebound_same_semantics']:
    other=Driver(name);initial=other.ok('show')
    assert initial['bundle_identity']!=before['bundle_identity']
    assert not other.call('restore',history=saved)['ok']
    assert other.ok('show')==initial;other.close()
report['raw_identity_rebinding_refused']=2

report['positive_boundaries']=[]
for name,turns in [('128_turns',[dict(speaker='SYSTEM',utterance='x')]*128),('4096_single_bytes',[dict(speaker='SYSTEM',utterance='é'*2048)]),('65536_total_bytes',[dict(speaker='SYSTEM',utterance='x'*4096)]*16)]:
    h=dict(saved,turns=turns);snap=d.ok('restore',history=h)
    assert snap['total_turns']==len(turns) and d.ok('save')['history']==h
    if name!='4096_single_bytes':
        assert not d.call('system',utterance='x')['ok']
        assert d.ok('show')==snap and d.ok('save')['history']==h
    report['positive_boundaries'].append(name)
d.ok('reset')
for text in ['', '  ', 'x\0y', 'é'*2049]:
    assert not d.call('user',utterance=text)['ok']
    assert d.ok('show')==empty

# Independently replay every permitted utterance-only DEV stream in round-robin order.
inputs=json.loads((ROOT/'draftsession-source/inputs.json').read_text())
expected=json.loads((ROOT/'draftsession-source/baseline-predictions.json').read_text())
actual=[[] for _ in inputs];restarts=0
for j in range(max(map(len,inputs))):
    for i,turns in enumerate(inputs):
        if j>=len(turns):continue
        t=turns[j];name='dev-'+str(i)
        snap=d.ok(t['speaker'].lower(),session=name,utterance=t['utterance'])
        if t['speaker']=='USER':actual[i].append(snap['draft'])
        if j==len(turns)//2:
            h=d.ok('save',session=name)['history']; assert d.ok('restore',session=name,history=h)==snap;restarts+=1
assert actual==expected,'Full DEV prediction mismatch'
report['full_dev']={'conversations':len(inputs),'user_predictions':sum(map(len,actual)),'interleaved':True,'restarts':restarts,'exact':True}
d.close()
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
