from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
a.out.mkdir(parents=True,exist_ok=False)
contracts=json.loads((HERE/'main-contracts.json').read_text());cases=json.loads((HERE/'cases.json').read_text())
expected={c['question']:r for c,r in zip(cases,map(json.loads,(HERE/'candidate/final-native.jsonl').read_text().splitlines()))}
report=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),modes=[])
def run(lines,path=None,model=None,human=False):
    cmd=[str(a.binary),'talk','--dialogue-dir',str(ROOT/'coordinator/draftsession-independent/bundles/valid'),'--facts-dir',str(ROOT/'coordinator/temporal-integrated/real-bundle')]
    if not human:cmd+=['--json']
    if path:cmd+=['--session',str(path)]
    if model:cmd+=['--model',model]
    r=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
    if human:return r.stdout
    rows=[json.loads(s) for s in r.stdout.splitlines()];assert len(rows)==len(lines),(lines,rows)
    with (a.out/'events.jsonl').open('a') as f:f.write(json.dumps(dict(lines=lines,rows=rows))+'\n')
    return rows
def state(path):return json.loads(path.read_text())
def preserved(before,after,line):
    assert after['turn']==before['turn']+1,(before,after)
    assert after['last_user_utterance']==line and after['last_slot'] is None
    ignored={'turn','last_user_utterance','last_slot'}
    assert {k:v for k,v in before.items() if k not in ignored}=={k:v for k,v in after.items() if k not in ignored},(line,before,after)
for label,model in [('plain',None),('intent','/home/gsc/.cache/glassmind/models/everyday/intent.json')]:
    path=a.out/(label+'.json')
    seed=run(['My name is Morgan','I like tea','Mary moved to the kitchen.','Draft request: I need a taxi to Cedar Museum.'],path,model)
    assert seed[-1]['status']=='updated' and 'request_draft' in state(path)
    count=0
    for c in cases:
        q=c['question']
        # Every case is routed explicitly; supported and reserved malformed forms also naturally.
        lines=[contracts['prefix']+q]
        if q in contracts['positive_natural'] or q in contracts['reserved_natural_malformed']:lines.insert(0,q)
        for line in lines:
            before=state(path);r=run([line],path,model)[0];preserved(before,state(path),line)
            assert r['kind']=='quantity_comparison',(line,r)
            wire={k:v for k,v in r.items() if k not in {'kind','text'}}
            assert wire==expected[q],(line,wire,expected[q]);count+=1
    for line in contracts['malformed_prefix']:
        before=state(path);r=run([line],path,model)[0];preserved(before,state(path),line)
        assert r['kind']=='quantity_comparison' and r['status']=='refusal';count+=1
    recall=run(['What is my name?','Where is Mary?','What is 2 plus 3?','Show request'],path,model)
    assert 'Morgan' in recall[0]['text'] and 'kitchen' in recall[1]['text']
    assert recall[2]['kind']=='calculated' and recall[2]['value']=='5'
    assert recall[3]['snapshot']==seed[-1]['snapshot']
    # Unsupported context references must remain explicit refusals, preserving every domain.
    for line in ['Compare quantities: Is my age greater than 2 years?','Compare quantities: Is it heavier than 1 kg?']:
        before=state(path);r=run([line],path,model)[0];preserved(before,state(path),line)
        assert r['status']=='refusal';count+=1
    report['modes'].append(dict(mode=label,comparison_commands=count,state_checks=count,restarted_after_every_comparison=True,recall_calc_draft_checks=4))
questions=['Is 1 pound greater than 0.5 kg?','Which is faster, 36 km/h or 10 m/s?','Which is longer, 3 feet or 1 metre?']
human=run(questions,human=True);lines=human.splitlines();assert lines[0].startswith('No.') and 'equal' in lines[1] and 'second quantity' in lines[2] and all('(exact)' in x for x in lines)
(a.out/'human.txt').write_text(human)
(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
