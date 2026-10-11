"""Independent explicit-route, persisted-state and full prediction checks."""
from pathlib import Path
import argparse,copy,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
HERE=Path(__file__).resolve().parent;ROOT=HERE.parent.parent
a.out.mkdir(parents=True,exist_ok=False)
contracts=json.loads((HERE/'main-contracts.json').read_text());expected=json.loads((HERE/'baseline-predictions.json').read_text())
report={'binary_sha256':hashlib.sha256(a.binary.read_bytes()).hexdigest(),'modes':[]}
def run(lines,session=None,bundle='valid',model=None,human=False):
    cmd=[str(a.binary),'talk']
    if not human:cmd+=['--json']
    if session:cmd+=['--session',str(session)]
    if bundle:cmd+=['--dialogue-dir',str(HERE/'bundles'/bundle)]
    if model:cmd+=['--model',model]
    result=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,timeout=120)
    assert result.returncode==0,(cmd,result.stderr)
    if human:return result.stdout
    out=[json.loads(x) for x in result.stdout.splitlines()];assert len(out)==len(lines),(lines,out)
    with (a.out/'events.jsonl').open('a') as f:f.write(json.dumps(dict(cmd=cmd,lines=lines,output=out),ensure_ascii=False)+'\n')
    return out
def state(path):return json.loads(path.read_text())
def preserved(before,after,line):
    assert after['turn']==before['turn']+1
    assert after['last_user_utterance']==line and after['last_slot'] is None
    for k in set(before)|set(after):
        if k not in {'turn','last_user_utterance','last_slot','request_draft'}:assert before.get(k)==after.get(k),(k,before,after)
for label,model in [('plain',None),('intent','/home/gsc/.cache/glassmind/models/everyday/intent.json')]:
    path=a.out/(label+'.json');count=0;states=0
    run(['My name is Morgan','I like tea','Mary moved to the kitchen.'],path,model=model)
    assert 'request_draft' not in state(path)
    for i,stream in enumerate(contracts['streams']):
        before=state(path);reply=run(['Reset request'],path,bundle=None,model=model)[0];after=state(path)
        preserved(before,after,'Reset request');assert reply['kind']=='dialogue_draft' and reply['status']=='reset' and 'request_draft' not in after;states+=1
        preds=[]
        for t in stream['turns']:
            line=contracts['prefixes'][t['speaker']]+t['utterance'];before=state(path)
            reply=run([line],path,model=model)[0];after=state(path);preserved(before,after,line);states+=1
            assert reply['kind']=='dialogue_draft' and reply['provisional']
            assert reply['status']==('updated' if t['speaker']=='USER' else 'context_recorded')
            if t['speaker']=='USER':preds.append(reply['snapshot']['draft'])
            show=run(['Show request'],path,model=model)[0]
            assert show['snapshot']==reply['snapshot'] and state(path).get('request_draft')==after.get('request_draft')
            count+=2
        assert preds==expected[i],stream['name']
    line=contracts['case_space_command'];r=run([line],path,model=model)[0];assert r['status']=='updated'
    assert state(path)['request_draft']['turns'][-1]['utterance']==contracts['case_space_payload']
    for line in contracts['malformed']:
        before=state(path);reply=run([line],path,model=model)[0];after=state(path)
        preserved(before,after,line);assert after['request_draft']==before['request_draft']
        assert reply['kind']=='dialogue_draft' and reply['status']=='refused';count+=1;states+=1
    for bundle in [None,'model_rebound_same_semantics','schema_rebound_same_semantics']:
        before=state(path);line='Draft request: I need a taxi.';r=run([line],path,bundle=bundle,model=model)[0]
        preserved(before,state(path),line);assert r['status']=='refused' and state(path)['request_draft']==before['request_draft'];count+=1;states+=1
    before=state(path)['request_draft']
    ordinary=['What is my name?','Where is Mary?','What is 2 plus 3?','Who is Ada Lovelace?','I like coffee']
    run(ordinary,path,model=model);assert state(path)['request_draft']==before
    # Corrupt saved history cannot replace prediction authority; explicit reset recovers.
    corrupted=state(path);corrupted['request_draft']['turns'][0]['frames']=[];path.write_text(json.dumps(corrupted))
    r=run(['Show request'],path,model=model)[0];assert r['status']=='refused' and state(path)['request_draft']==corrupted['request_draft']
    run(['Reset request'],path,bundle=None,model=model);assert 'request_draft' not in state(path)
    r=run(['Show request'],path,model=model)[0];assert r['snapshot']['draft'] is None and 'request_draft' not in state(path)
    run(['Draft request: I need a taxi.','/reset','Show request'],path,model=model);assert 'request_draft' not in state(path)
    report['modes'].append(dict(mode=label,commands=count,state_checks=states,ordinary_isolated=len(ordinary),process_restart_after_every_command=True))

inputs=json.loads((ROOT/'draftsession-source/inputs.json').read_text());preds=json.loads((ROOT/'draftsession-source/baseline-predictions.json').read_text())
lines=[];wanted=[]
for turns,expect in zip(inputs,preds):
    lines.append('Reset request');wanted.append(None);it=iter(expect)
    for t in turns:
        lines.append(contracts['prefixes'][t['speaker']]+t['utterance']);wanted.append(next(it) if t['speaker']=='USER' else None)
outputs=run(lines)
for row,exp in zip(outputs,wanted):
    assert row['kind']=='dialogue_draft' and row['provisional'] and row['status']!='refused'
    if exp is not None:assert row['snapshot']['draft']==exp
report['full_dev_user_predictions']=sum(x is not None for x in wanted)
report['full_dev_conversations']=len(inputs)
human=run(['Show request','Draft request: I need a taxi to Cedar Museum.','Service reply: How many passengers?'],human=True)
assert 'Provisional request draft' in human and 'provisional request draft' in human
(a.out/'human.txt').write_text(human)
path=a.out/'plain.json';before=path.read_bytes()
invalid=subprocess.run([str(a.binary),'talk','--session',str(path),'--dialogue-dir',str(HERE/'bundles/model_hash_mismatch')],input='Reset request\n',text=True,capture_output=True,timeout=30)
assert invalid.returncode!=0 and not invalid.stdout and path.read_bytes()==before
report['invalid_declared_bundle_fails_without_mutation']=True
(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
