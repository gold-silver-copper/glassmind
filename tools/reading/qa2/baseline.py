import collections,hashlib,importlib.util,json,os,subprocess
from pathlib import Path
w=Path('/home/gsc/.cache/glassmind/continuous-20261010')
src=w/'readingqa2-source';run=w/'coordinator/qa2-baseline';run.mkdir(exist_ok=True)
os.environ['SQUAD_SOURCE']=str(src)
sp=importlib.util.spec_from_file_location('qa','tools/reading/qa/run.py');qa=importlib.util.module_from_spec(sp);sp.loader.exec_module(qa)
binary=Path('target/release/reading-qa').resolve();model=w/'qaroles-run/baseline/model.json'
assert hashlib.sha256(model.read_bytes()).hexdigest()=='b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5'
prior=json.loads((run/'evidence.json').read_text()) if (run/'evidence.json').exists() else None
if prior:
 assert prior['model_sha256']==hashlib.sha256(model.read_bytes()).hexdigest()
 assert prior['binary_sha256']==hashlib.sha256(binary.read_bytes()).hexdigest()
 for artifact in prior['artifacts']:
  saved=Path(artifact['path']);assert saved.stat().st_size==artifact['bytes'] and hashlib.sha256(saved.read_bytes()).hexdigest()==artifact['sha256']
elif any(run.glob('*.outputs.jsonl')):raise RuntimeError('Existing predictions require frozen evidence identities before reuse')
rows=[dict(id=q['id'],context=p['context'],question=q['question'],answers=q['answers'],impossible=q['is_impossible']) for a in json.loads((src/'dev-v2.0.json').read_text())['data'] for p in a['paragraphs'] for q in p['qas']]
rows.sort(key=qa.order);assert len(rows)==len({r['id'] for r in rows})==11873
assert sum(r['impossible'] for r in rows)==5945
reports={}
for name,rs in [('sample',rows[:200]),('full',rows)]:
 inp=run/(name+'.input.jsonl');out=run/(name+'.outputs.jsonl');expected_input=''.join(json.dumps(dict(context=r['context'],question=r['question']))+'\n' for r in rs)
 if out.exists(): assert inp.read_text()==expected_input
 else: inp.write_text(expected_input)
 if not out.exists():
  with out.open('w') as f:subprocess.run([str(binary),'predict','--input',str(inp),'--model',str(model)],stdout=f,check=True,timeout=550)
 ps=[json.loads(s) for s in out.read_text().splitlines()];assert len(ps)==len(rs)
 selected={r['id']:p['answer'] for r,p in zip(rs,ps,strict=True)};raw={r['id']:qa.proposal(r,p) for r,p in zip(rs,ps,strict=True)}
 dataset=[dict(paragraphs=[dict(qas=[dict(id=r['id'],answers=r['answers']) for r in rs])])]
 ex,f1=qa.SCORE['get_raw_scores'](dataset,selected);rex,rf1=qa.SCORE['get_raw_scores'](dataset,raw)
 assert set(ex)==set(f1)==set(selected)==set(raw)=={r['id'] for r in rs}
 counts=collections.Counter(); per=[]; coincidences=[]
 for r,p in zip(rs,ps,strict=True):
  if p['status']=='refused':status='correct_expected_refusal' if r['impossible'] else 'unsupported_answerable'
  else:status='correct_substantive' if ex[r['id']] and not r['impossible'] else 'wrong'
  counts[status]+=1;per.append(dict(id=r['id'],impossible=r['impossible'],status=status,official_exact=ex[r['id']]))
  substantive_or_refusal=status in ('correct_substantive','correct_expected_refusal')
  if bool(ex[r['id']]) != substantive_or_refusal:
   assert p['status']=='predicted' and r['impossible'] and qa.SCORE['normalize_answer'](p['answer'])=='' and ex[r['id']]==1
   coincidences.append(dict(id=r['id'],answer=p['answer'],note='Official normalization treats this substantive prediction as empty on an unanswerable item; diagnostic retains it as wrong.'))
 assert sum(counts.values())==len(rs);assert sum(ex.values())==counts['correct_substantive']+counts['correct_expected_refusal']+len(coincidences)
 metrics=dict(total=len(rs),counts=dict(counts),normalization_coincidences=coincidences,official_selected=qa.SCORE['make_eval_dict'](ex,f1),official_raw=qa.SCORE['make_eval_dict'](rex,rf1),all_refusal_exact=sum(r['impossible'] for r in rs),answerable=sum(not r['impossible'] for r in rs),selected_precision=counts['correct_substantive']/(counts['correct_substantive']+counts['wrong']) if counts['correct_substantive']+counts['wrong'] else None)
 (run/(name+'.report.json')).write_text(json.dumps(metrics,indent=2)+'\n');(run/(name+'.statuses.json')).write_text(json.dumps(per,indent=2)+'\n');reports[name]=metrics;print(name,json.dumps(metrics),flush=True)
metadata=dict(build_source=prior['build_source'] if prior else subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),reading_runtime='unchanged v1 proposal and margin selector; not a new model',source_revision='eee5fdbf62f8613a7812b03419e6b29617b74fd1',model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),scorer_sha256=hashlib.sha256((src/'evaluate-v2.0.py').read_bytes()).hexdigest(),reports=reports,reused_complete_predictions_after_diagnostic_assertion='Original count check caught one normalization coincidence; both sample/full outputs reused, no repeated inference.',selection='SHA256(question ID), first200 then all11873; IDs and annotations never enter inference input',limitations='New publicDEV dataset/metric series; expected correct refusals separate from substantive answers; all-refusal comparator reported. No test or external predictions. No fitting/calibration/candidateevaluation yet.',artifacts=[dict(path=str(p),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in run.iterdir() if p.is_file() and p.name!='evidence.json'])
(run/'evidence.json').write_text(json.dumps(metadata,indent=2)+'\n')
