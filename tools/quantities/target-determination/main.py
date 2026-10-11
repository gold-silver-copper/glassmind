from pathlib import Path
from fractions import Fraction
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent
WORK = HERE.parents[1]
binary = HERE/'glassmind'
cases = json.loads((HERE/'cases.json').read_text())
out = HERE/'main'
out.mkdir(exist_ok=False)
reports = []
for mode, model in [('plain',[]),('intent',['--model','/home/gsc/.cache/glassmind/models/everyday/intent.json'])]:
    path = out/(mode+'.json')
    cmd = [str(binary),'talk','--json','--session',str(path),
           '--facts-dir',str(WORK/'coordinator/temporal-integrated/real-bundle'),
           '--dialogue-dir',str(WORK/'coordinator/draftsession-independent/bundles/valid')]+model
    def run(lines):
        r = subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60)
        rows = [json.loads(x) for x in r.stdout.splitlines()]
        assert len(rows)==len(lines)
        return rows
    seed = run(['My name is Morgan','I like tea','Mary moved to the kitchen.','Draft request: I need a taxi to Cedar Museum.'])
    assert seed[-1]['status']=='updated'
    initial = path.read_bytes()
    state = json.loads(initial)
    results = []
    for case in cases:
        path.write_bytes(initial)
        reply = run([case['question']])[0]
        if case['expected'] is None:
            assert reply['kind']!='calculated', (case,reply)
        else:
            assert reply['kind']=='calculated' and reply.get('unit') is None and Fraction(reply['value'])==Fraction(case['expected']), (case,reply)
            after = json.loads(path.read_bytes())
            assert after['turn']==state['turn']+1 and after['last_user_utterance']==case['question']
            ignored = {'turn','last_user_utterance','last_slot'}
            assert {k:v for k,v in state.items() if k not in ignored}=={k:v for k,v in after.items() if k not in ignored}
            recall = run(['What is my name?','Where is Mary?','Show request'])
            assert 'Morgan' in recall[0]['text'] and 'kitchen' in recall[1]['text']
            assert recall[2]['snapshot']==seed[-1]['snapshot']
        results.append(dict(question=case['question'],reply=reply,passed=True))
    reports.append(dict(mode=mode,total=len(results),passed=len(results),positive_state_and_recall_checks=sum(c['expected'] is not None for c in cases),cases=results))
report = dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),modes=reports)
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps([{k:v for k,v in r.items() if k!='cases'} for r in reports]))
