"""First200/full2821 exact legacy replay; retained baseline, no repeated baseline inference."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--predictor',type=Path,required=True);p.add_argument('--baseline',type=Path,required=True);p.add_argument('--baseline-binary',type=Path,required=True);p.add_argument('--data',type=Path,required=True);p.add_argument('--index',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=True)
rows=[r.split('\t',3) for r in (a.data/'valid.tsv').read_text().splitlines()];assert len(rows)==2821
baseline=a.baseline.read_bytes().splitlines(keepends=True);assert len(baseline)==2821
reports=[]
for count in (200,2821):
    raw=subprocess.run([a.predictor,a.data/'relation.json',a.index],input=''.join(r[3]+'\n' for r in rows[:count]).encode(),capture_output=True,check=True,timeout=550).stdout
    (a.output/f'standalone-{count}.jsonl').write_bytes(raw)
    assert raw==b''.join(baseline[:count]),count
    predictions=[json.loads(line) for line in raw.splitlines()]
    right=refused=wrong=joint=0
    for row,ans in zip(rows,predictions):
        s,p,o,_=row
        if ans['answers']==[o]:
            right+=1;joint+=int(ans['entity']==s and ans['relation']==p)
        elif ans['refusal'] is not None: refused+=1
        else: wrong+=1
    score_raw=subprocess.run([a.binary,'score','--input',a.data/'valid.tsv','--model',a.data/'relation.json','--index',a.index,'--limit',str(count)],capture_output=True,check=True,timeout=550).stdout
    (a.output/f'native-score-{count}.json').write_bytes(score_raw)
    baseline_score=subprocess.run([a.baseline_binary,'score','--input',a.data/'valid.tsv','--model',a.data/'relation.json','--index',a.index,'--limit',str(count)],capture_output=True,check=True,timeout=550).stdout
    (a.output/f'baseline-native-score-{count}.json').write_bytes(baseline_score)
    assert score_raw==baseline_score,('native scorer bytes',count)
    score=json.loads(score_raw);assert (score['right'],score['refused'],score['wrong'])==(right,refused,wrong)
    report=dict(rows=count,byte_identical=True,native_scorer_byte_identical=True,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),answer_only=dict(right=right,refused=refused,wrong=wrong),joint=dict(right=joint,refused=refused,wrong=count-joint-refused),native_score=score)
    if count==2821: assert (right,refused,wrong,joint)==(256,2545,20,252)
    reports.append(report)
(a.output/'report.json').write_text(json.dumps(reports,indent=2)+'\n');print(json.dumps(reports,indent=2))
