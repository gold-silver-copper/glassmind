"""Causal analysis of the rejected TRAIN calibration result, no DEV tuning."""
import json,pathlib,os,collections,hashlib
run=pathlib.Path(os.environ['TMPDIR'])/'spansequence';r=json.loads((run/'train-report.json').read_text())
states={}
root=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full')
for d in json.loads((root/'selected/train.json').read_text()):
 if int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5:continue
 system=''
 for t in d['turns']:
  if t['speaker']=='SYSTEM':system=t['utterance'];continue
  for f in t['frames']:
   states[t['utterance'],system,f['service']]=dict(dialogue_id=d['dialogue_id'],state=f.get('state'))
counts=collections.Counter();errors=[];gold_total=0;high_right=0
for row in r['calibration_rows']:
 gold_total+=len(row['gold'])
 for p in row['proposals']:
  match=any(g['slot']==p['slot'] and g['start']==p['char_start'] and g['exclusive_end']==p['char_end'] for g in row['gold'])
  if match:
   if p['margin']>=32:high_right+=1
   continue
  gs=[g for g in row['gold'] if g['slot']==p['slot']];overlap=[g for g in gs if g['start']<p['char_end'] and g['exclusive_end']>p['char_start']]
  reason='boundary mismatch' if overlap else 'unannotated slot proposal'
  record=dict(service=row['service'],utterance=row['utterance'],system=row['system'],proposal=p,gold=[dict(g,value=row['utterance'][g['start']:g['exclusive_end']]) for g in row['gold']],reason=reason)
  state=states.get((row['utterance'],row['system'],row['service']),{})
  record.update(state)
  gold_values=(state.get('state') or {}).get('slot_values',{}).get(p['slot'],[])
  record['matches_current_gold_value']=p['value'].lower() in [v.lower() for v in gold_values]
  errors.append(record)
  if p['margin']>=32:counts[reason]+=1
summary=dict(decision='rejected CAL before candidate DEV',raw_proposals=r['cal_proposals_correct'][0],highest_threshold_proposals=r['cal_proposals_correct'][-1],gold_spans=gold_total,high_threshold_gold_recall=high_right/gold_total,high_threshold_errors=counts,all_raw_errors=errors,highest_threshold_errors=[e for e in errors if e['proposal']['margin']>=32],schedule_rerun_to_preserve_failed_weights=True,highest_errors_state_match=collections.Counter('matches' if e['matches_current_gold_value'] else 'differs' for e in errors if e['proposal']['margin']>=32))
(run/'cal-audit.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:v for k,v in summary.items() if not isinstance(v,list)},indent=2))
for e in summary['highest_threshold_errors']:print(json.dumps(e,ensure_ascii=False))
