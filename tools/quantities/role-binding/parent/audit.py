"""Independent rejection audit: original TRAIN labels, exact native replay and gates."""
from pathlib import Path
import collections,hashlib,json,subprocess,sys
import pyarrow.parquet as pq
w=Path('/home/gsc/.cache/glassmind/continuous-20261010');out=Path(__file__).parent;f=w/'rolebinding-run/frozen-candidate';e=f/'evidence';src=w/'groundingaudit-source';read=lambda p:[json.loads(s) for s in p.read_text().splitlines()]
manifest=json.loads((f/'manifest.json').read_text())
for name,r in manifest.items():
 b=(f/name).read_bytes();assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256']
gold=read(e/'cal-gold.jsonl');prior=[r for r in read(w/'groundingaudit-run/frozen-candidate/audit/clean.jsonl') if r['split']=='CAL'];assert len(gold)==len(prior)==1182
assert [{'index':r['id'],'question':r['question'],'program':r['program']} for r in gold]==[{k:r[k] for k in ['index','question','program']} for r in prior]
labels=pq.read_table(src/'train.parquet',columns=['Problem','options','correct']).to_pylist();assert len(labels)==29837
for g in gold:assert g['question']==labels[g['id']]['Problem']
actual=read(out/'cal-replay.jsonl');assert actual==read(e/'cal-candidate-free.jsonl');assert read(e/'cal-original-native-free.jsonl')==read(e/'cal-baseline-free.jsonl')
assert read(e/'baseline200-native.jsonl')==read(e/'baseline200-final-helper.jsonl')
metric=Path('/home/gsc/code/glassmind/tools/quantities/typed-label-metric/metric.py');assert hashlib.sha256(metric.read_bytes()).hexdigest()=='24f1d65ed2f1785b940d7c674565dd9b49c168e498f57c23cbd6c8c2c6eab6f0';sys.path.insert(0,str(metric.parent));from metric import compare,parse_gold
totals={};grades={};native_inputs=[];native_expected=[]
for model in ['baseline','candidate']:
 totals[model]={}
 for mode in ['free','oracle']:
  pred=read(e/f'cal-{model}-{mode}.jsonl');assert len(pred)==len(gold);count=collections.Counter();graded=[]
  for g,p in zip(gold,pred,strict=True):
   lab=labels[g['id']];answer=dict(status='answer',value=p['value'],unit=None) if p['status']=='proposed' else dict(status='refusal');grade=compare(parse_gold(lab['options'],lab['correct'])[1],answer)['grade'];count[grade]+=1;count['exact_ast']+=p.get('program')==g['program'];graded.append(grade)
   if model=='candidate' and mode=='free' and p['status']=='proposed':native_inputs.append(dict(question=g['question'],program=p['program']));native_expected.append(p)
  totals[model][mode]=dict(count);grades[model,mode]=graded
 teacher=read(e/f'cal-{model}-teacher.jsonl');counts=collections.Counter();arg_id=lambda a:{'input':0,'constant':16,'step':36}[a['kind']]+a['index']
 for g,t in zip(gold,teacher,strict=True):
  assert g['id']==t['id'];assert len(g['program']['steps'])==len(t['steps']);row=[]
  for s,t in zip(g['program']['steps'],t['steps'],strict=True):
   assert t['gold_left']==arg_id(s['left']) and t['gold_right']==arg_id(s['right']);assert t['op']==['add','subtract','multiply','divide'].index(s['op']);counts['decisions']+=1;counts['op_correct']+=t['op_correct'];counts['left_correct']+=t['left']==t['gold_left'];counts['right_correct']+=t['right']==t['gold_right'];row.append(t['left']==t['gold_left'] and t['right']==t['gold_right'])
  counts['whole_arguments']+=all(row)
 totals[model]['teacher']=dict(counts)
payload=''.join(json.dumps(r)+'\n' for r in native_inputs);p=subprocess.run([str(src/'equation_executor')],input=payload,text=True,capture_output=True,timeout=60);assert p.returncode==0,p.stderr;native=[json.loads(s) for s in p.stdout.splitlines()];assert len(native)==len(native_expected)==1181
for actual,pred in zip(native,native_expected):
 for k in ['status','numbers','program','value']:assert actual[k]==pred[k]
(out/'native-cal-proposed.jsonl').write_text(p.stdout)
gates=json.loads((out/'gates.json').read_text())['required'];candidate=totals['candidate'];checks=dict(left=candidate['teacher']['left_correct']>=gates['left_correct_min'],right=candidate['teacher']['right_correct']>=gates['right_correct_min'],free=candidate['free']['correct']>=gates['free_numeric_min'],oracle=candidate['oracle']['correct']>=gates['oracle_numeric_min']);assert checks==dict(left=True,right=False,free=False,oracle=False)
transitions=collections.Counter(a+'->'+b for a,b in zip(grades['baseline','free'],grades['candidate','free']))
report=dict(decision='rejected',frozen_manifest_entries=len(manifest),clean_cal_membership_exact=1182,native_free_replay_exact=1182,independent_native_proposed=1181,baseline200_exact=True,original_comparator_sha256=hashlib.sha256(metric.read_bytes()).hexdigest(),totals=totals,gates=checks,free_transitions=dict(transitions),scope='TRAIN calibration, annotation-cleaned, oracle modes explicitly conditioned; no DEV or production acceptance')
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
