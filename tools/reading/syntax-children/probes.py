"""Frozen authored core-edge retention; raw contrasts and span checks."""
import json,pathlib,subprocess,sys
baseline,out=map(pathlib.Path,sys.argv[1:])
def read(p): return [json.loads(x) for x in p.read_text().splitlines()]
cases=read(pathlib.Path('tools/reading/syntax-children/authored.jsonl'))
legacy=read(pathlib.Path('tools/reading/syntax/authored.jsonl'))
inputs=[{'forms':c['forms']} for c in cases]+[{'text':c['text']} for c in legacy]
core=[c['core_edges'] for c in cases]+[c.get('core',[]) for c in legacy]
raw=read(pathlib.Path('tools/reading/raw-syntax/authored.jsonl'))
inputs += [{'text':c['text']} for c in raw]
inp=out/'probe-input.jsonl';inp.write_text(''.join(json.dumps(i)+'\n' for i in inputs))
ps=[]
for name,model in [('old',baseline/'model.bin'),('new',out/'model.bin')]:
 pred=out/f'probe-{name}.jsonl'
 subprocess.run([str(out/'reading-syntax'),'predict','--model',str(model),'--input',str(inp),'--output',str(pred)],check=True)
 ps.append(read(pred))
ledger=[]
for i,edges in enumerate(core):
 for d,h,l in edges:
  correct=[p[i]['heads'][d-1]==h and p[i]['deprels'][d-1]==l for p in ps]
  ledger.append(dict(case=i,edge=[d,h,l],before=correct[0],after=correct[1]))
for predictions in ps:
 for c,p in zip(raw,predictions[len(core):]):
  assert p['forms']==c['forms']
  data=c['text'].encode();end=0
  for span in p['spans']:
   assert span['start']>=end and data[span['start']:span['end']].decode()==span['form']
   assert not data[end:span['start']].decode().strip()
   end=span['end']
  assert not data[end:].decode().strip()
r={'new_authored_cases':len(cases),'legacy_authored_cases':len(legacy),'raw_contrasts':len(raw),'core_edges':len(ledger),'before_correct':sum(x['before'] for x in ledger),'after_correct':sum(x['after'] for x in ledger),'lost':sum(x['before'] and not x['after'] for x in ledger),'gained':sum(not x['before'] and x['after'] for x in ledger),'ledger':ledger}
(out/'probes-report.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k!='ledger'},indent=2))
