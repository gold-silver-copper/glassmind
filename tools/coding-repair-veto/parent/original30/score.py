from pathlib import Path
import json,sys
base=Path(__file__).parent
for name in sys.argv[1:]:
 d=base/name;rows=json.loads((d/'report.json').read_text())['cases'];assert len(rows)==60
 positives=[x for x in rows if x['expected']=='repair_build' and x['mode']=='do']
 statuses=[x for x in rows if x['expected']=='status' and x['mode']=='do']
 def status(x,v):
  kind='Problems' if v==2 and x['name']=='status_2' else 'Passes'
  return x['preview_meanings']==[f"Ask({kind}({x['check']}))"]
 report=dict(positive_do_exact_native=sum(x['changed'] and x['exact_repair'] and x['native_exit']==0 for x in positives),positive_denominator=8,unrequested_edits=sum(x['changed'] for x in rows if x['expected']!='repair_build'),negative_commands=sum(x['preview_mutating'] for x in rows if x['expected']=='no_mutating_action'),ask_apply_unchanged=all(not x['changed'] for x in rows if x['mode']=='ask'),status_v1=sum(status(x,1) for x in statuses),status_v2=sum(status(x,2) for x in statuses),status_denominator=6,version='separate explicit public status type correction; originals preserved')
 (d/'status-contract-v2-summary.json').write_text(json.dumps(report,indent=2)+'\n');print(name,json.dumps(report))
