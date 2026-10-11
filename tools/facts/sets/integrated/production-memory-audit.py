from pathlib import Path
import ast,hashlib,json,subprocess
R=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator/factsets-check');OUT=R/'memory-model-compatibility';OUT.mkdir();W=Path('/home/gsc/.cache/glassmind/continuous-20261010/factsets-run/tmp/sets')
BASE=W/'baseline/glassmind';CAND=Path('/home/gsc/code/glassmind/target/release/glassmind');MODEL=Path('/home/gsc/.cache/glassmind/models/everyday/intent.json');SOURCE=Path('crates/everyday/tools/smoke_memory.py');BUNDLE=W/'bundle'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(BASE)=='0d5c9ea8be0e4d3c83b17508e5666ed814bb8b8f4e78364f478fd7d3e3a0ba21'
module=ast.parse(SOURCE.read_text());main=next(n for n in module.body if isinstance(n,ast.FunctionDef) and n.name=='main');values=next(ast.literal_eval(n.value) for n in main.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='cases' for t in n.targets));groups=[]
for n in main.body:
 if isinstance(n,ast.Expr) and isinstance(n.value,ast.Call) and isinstance(n.value.func,ast.Name) and n.value.func.id=='run':
  args=n.value.args;rows=values if isinstance(args[0],ast.Name) else ast.literal_eval(args[0]);groups.append((rows,len(args)>1))
assert len(groups)==6 and sum(len(rows) for rows,_ in groups)==40
reports=[]
for facts in [False,True]:
 rawsets={};states={}
 for name,binary in [('baseline',BASE),('candidate',CAND)]:
  folder=OUT/(name+('-facts' if facts else '-default'));folder.mkdir();session=folder/'session.json';rawsets[name]=[];mismatches=[];number=0;states[name]=[]
  for g,(rows,persist) in enumerate(groups):
   cmd=[str(binary),'talk','--json','--model',str(MODEL)]+(['--facts-dir',str(BUNDLE)] if facts else [])+(['--session',str(session)] if persist else [])
   raw=subprocess.run(cmd,input='\n'.join(r[0] for r in rows)+'\n',text=True,capture_output=True,check=True,timeout=180).stdout
   (folder/(str(g)+'.jsonl')).write_text(raw);rawsets[name].append(raw);replies=[json.loads(l) for l in raw.splitlines()];assert len(rows)==len(replies)
   for (utterance,kind,value,turn),reply in zip(rows,replies,strict=True):
    number+=1
    if reply['kind']!=kind or (value and value not in reply.get('text','')) or (turn is not None and reply.get('source_turn')!=turn):mismatches.append(dict(row=number,utterance=utterance,expected_kind=kind,actual=reply))
   if persist:states[name].append(json.loads(session.read_text()))
   if g==2:
    state=json.loads(session.read_text());assert state['last_correction']['previous']['value']=='Café con leche' and state['preferences']['drink']['value']=='Oolong'
  reports.append(dict(binary=name,facts_enabled=facts,total_expectations=41,matched_expectations=41-len(mismatches),mismatches=mismatches))
 assert rawsets['baseline']==rawsets['candidate'] and states['baseline']==states['candidate']
report={'source_sha256':sha(SOURCE),'baseline_sha256':sha(BASE),'candidate_sha256':sha(CAND),'model_sha256':sha(MODEL),'all_six_conversation_streams_byte_identical_each_mode':True,'persisted_states_identical':True,'scope':'Additional production-model compatibility audit against unchanged original expectations. Original no-model smoke remains unchanged; mismatches are recorded, not waived or relabeled as passes. Runtime receives only utterances.','results':reports};(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'all_bytes_and_states_identical':True,'results':[{'binary':r['binary'],'facts':r['facts_enabled'],'matches':r['matched_expectations'],'total':41,'mismatch_rows':[m['row'] for m in r['mismatches']]} for r in reports]}))
