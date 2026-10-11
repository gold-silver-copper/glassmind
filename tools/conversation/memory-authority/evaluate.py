"""Frozen product contracts and unchanged original memory smoke expectations."""
import argparse,ast,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--model',type=Path);p.add_argument('--sample',action='store_true');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
checks=[];mode='model' if a.model else 'plain'
def check(name,expected,actual,ok):checks.append(dict(name=name,expected=expected,actual=actual,right=bool(ok)))
def run(rows,session,name):
    cmd=[str(a.binary.resolve()),'talk','--json']+(['--model',str(a.model)] if a.model else [])+(['--session',str(session)] if session else [])
    r=subprocess.run(cmd,input='\n'.join(r[0] for r in rows)+'\n',capture_output=True,text=True,check=True,timeout=60)
    (a.output/(name+'.jsonl')).write_text(r.stdout)
    replies=[json.loads(l) for l in r.stdout.splitlines()];assert len(replies)==len(rows)
    for i,((text,e),actual) in enumerate(zip(rows,replies,strict=True)):
        e={k:(v[mode] if isinstance(v,dict) else v) for k,v in e.items()}
        ok=all((v in actual.get('text','')) if k=='contains' else actual.get(k)==v for k,v in e.items())
        check(name+':'+str(i)+':'+text,e,actual,ok)
def state_checks(path,expect,name):
    state=json.loads(path.read_text())
    for key,value in expect.items():
        actual=state
        for part in key.split('.'):actual=actual.get(part) if isinstance(actual,dict) else None
        check(name+':'+key,value,actual,actual==value)
    (a.output/(name+'-state.json')).write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n')
contracts=json.loads((Path(__file__).parent/'contracts.json').read_text())
for n,s in enumerate(contracts[:4] if a.sample else contracts):
    path=a.output/f'contract{n}-session.json';path.unlink(missing_ok=True)
    if 'initial' in s:path.write_text(json.dumps(s['initial']))
    for j,phase in enumerate(s['phases']):
        name=f'contract{n}-{j}';run(phase['rows'],path,name);state_checks(path,phase['state'],name)
contract_total=len(checks)
if not a.sample:
    source=Path('crates/everyday/tools/smoke_memory.py');tree=ast.parse(source.read_text());main=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='main')
    cases=next(ast.literal_eval(n.value) for n in main.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='cases' for t in n.targets))
    groups=[n.value for n in main.body if isinstance(n,ast.Expr) and isinstance(n.value,ast.Call) and isinstance(n.value.func,ast.Name) and n.value.func.id=='run'];assert len(groups)==6
    session=a.output/'original-session.json';session.unlink(missing_ok=True)
    for i,g in enumerate(groups):
        rows=cases if isinstance(g.args[0],ast.Name) else ast.literal_eval(g.args[0]);converted=[]
        for text,kind,value,turn in rows:
            expected={'kind':kind}
            if value:expected['contains']=value
            if turn is not None:expected['source_turn']=turn
            converted.append([text,expected])
        run(converted,session if len(g.args)>1 else None,f'original{i}')
        if i==2:
            state=json.loads(session.read_text());ok=state['last_correction']['previous']['value']=='Café con leche' and state['preferences']['drink']['value']=='Oolong';check('original-provenance',True,ok,ok)
report=dict(mode=mode,sample=a.sample,contracts=dict(right=sum(c['right'] for c in checks[:contract_total]),total=contract_total),original=dict(right=sum(c['right'] for c in checks[contract_total:]),total=len(checks)-contract_total),checks=checks)
(a.output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='checks'}))
