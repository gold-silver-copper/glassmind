from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--model',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--standalone',action='store_true');a=p.parse_args();base=Path(__file__).parent
raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];cases=json.loads(raw);a.out.mkdir(parents=True,exist_ok=False)
invalid=[dict(text=''),dict(text='x'*16385),dict(text=' '.join(['word']*129)),dict(text='a\x00b'),dict(text='hello',forms=['hello']),dict(text='hello',heads=[0])]
inputs=[dict(text=c['text']) for c in cases]+invalid;f=a.out/'inputs.jsonl';f.write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in inputs))
cmd=[str(a.binary.resolve()),'predict' if a.standalone else 'syntax','--model',str(a.model.resolve()),'--input',str(f.resolve())]
if a.standalone:cmd+=['--output',str((a.out/'predictions.jsonl').resolve())]
r=subprocess.run(cmd,text=True,capture_output=True,timeout=120);(a.out/'stderr.log').write_text(r.stderr);assert r.returncode==0
if not a.standalone:(a.out/'predictions.jsonl').write_text(r.stdout)
pred=[json.loads(l) for l in (a.out/'predictions.jsonl').read_text().splitlines()];assert len(pred)==len(inputs);rows=[]
for c,v in zip(cases,pred):
 assert v['status']=='predicted',c
 spans=v['spans'];rawtext=c['text'].encode();last=0
 for t in spans:
  assert type(t['start']) is int and type(t['end']) is int and last<=t['start']<t['end']<=len(rawtext)
  assert rawtext[t['start']:t['end']].decode()==t['form'];last=t['end']
 assert [t['form'] for t in spans]==v['forms']
 assert ''.join(t['form'] for t in spans)==''.join(ch for ch in c['text'] if not ch.isspace())
 rows.append(dict(name=c['name'],expected=c['expected'],actual=v['forms'],exact=c['expected']==v['forms'] if c['expected'] is not None else None,valid_spans=True))
assert all(v['status']=='refused' for v in pred[len(cases):])
report=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(a.model.read_bytes()).hexdigest(),positive_correct=sum(c['exact'] is True for c in rows),source_span_cases=len(rows),invalid_refusals=len(invalid),cases=rows)
(a.out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'},indent=2))
