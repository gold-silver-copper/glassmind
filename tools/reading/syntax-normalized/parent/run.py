from pathlib import Path
import argparse,hashlib,importlib.util,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--model',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--beam',action='store_true');a=p.parse_args();base=Path(__file__).parent;raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];cases=json.loads(raw);a.out.mkdir(parents=True,exist_ok=False);f=a.out/'inputs.jsonl';f.write_text(''.join(json.dumps(dict(forms=c['forms']))+'\n' for c in cases));out=a.out/'predictions.jsonl'
cmd=[str(a.binary.resolve()),'predict','--model',str(a.model.resolve()),'--input',str(f.resolve()),'--output',str(out.resolve())]
if a.beam:cmd+=['--normalized-beam']
r=subprocess.run(cmd,text=True,capture_output=True,timeout=120);(a.out/'stdout.log').write_text(r.stdout);(a.out/'stderr.log').write_text(r.stderr);assert r.returncode==0
spec=importlib.util.spec_from_file_location('tree_audit','/home/gsc/code/glassmind/tools/reading/syntax/parent/audit.py');tree=importlib.util.module_from_spec(spec);spec.loader.exec_module(tree)
pred=[json.loads(l) for l in out.read_text().splitlines()];assert len(pred)==len(cases);rows=[]
for c,v in zip(cases,pred):
 assert v['status']=='predicted' and v['forms']==c['forms'] and tree.is_tree(v['heads'],v['deprels'])
 edges=[dict(**e,correct=v['heads'][e['dependent']-1]==e['head'] and v['deprels'][e['dependent']-1]==e['label'],actual_head=v['heads'][e['dependent']-1],actual_label=v['deprels'][e['dependent']-1]) for e in c['edges']];rows.append(dict(name=c['name'],edges=edges,all_correct=all(e['correct'] for e in edges)))
report=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(a.model.read_bytes()).hexdigest(),beam=a.beam,sentences=20,correct_sentences=sum(c['all_correct'] for c in rows),core_edges=40,correct_edges=sum(e['correct'] for c in rows for e in c['edges']),cases=rows);(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'},indent=2))
