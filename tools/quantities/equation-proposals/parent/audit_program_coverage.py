from pathlib import Path
import collections,json,sys
import pyarrow.parquet as pq
root=Path('/home/gsc/code/glassmind');work=Path('/home/gsc/.cache/glassmind/continuous-20261010');run=work/'equationproposals-run/frozen-candidate'
sys.path.insert(0,str(root/'tools/quantities/typed-label-metric'));from metric import parse_gold,compare
split=json.loads((run/'split.json').read_text());outs=[json.loads(l) for l in (run/'train-executions.jsonl').read_text().splitlines()];labels=pq.read_table(work/'equation-source/train.parquet',columns=['options','correct']).to_pylist()
key=lambda p:json.dumps(p,sort_keys=True,separators=(',',':'))
def good(i):
 p=outs[i]
 if p['status']!='proposed':return False
 _,gold=parse_gold(labels[i]['options'],labels[i]['correct'])
 return compare(gold,dict(status='answer',value=p['value'],unit=None))['grade']=='correct'
fit=[i for i in split['fit'] if good(i)];cal=[i for i in split['calibration'] if good(i)];bank={key(outs[i]['program']) for i in fit};candidate={key(c['program']) for c in json.loads((run/'model.json').read_text())['classes']}
c=collections.Counter();by_steps=collections.defaultdict(collections.Counter)
for i in cal:
 p=outs[i]['program'];s=key(p);kind='candidate_bank' if s in candidate else 'fit_only_bank' if s in bank else 'unseen_program';c[kind]+=1;by_steps[len(p['steps'])][kind]+=1
report=dict(scope='TRAIN calibration annotation coverage only; no DEV formulas read. Exact generic program identity is not equivalence or model performance.',clean_fit=len(fit),clean_calibration=len(cal),all_calibration=len(split['calibration']),coverage=dict(c),by_steps=dict(by_steps))
p=Path(__file__).parent/'candidate-review/program-coverage.json';p.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
