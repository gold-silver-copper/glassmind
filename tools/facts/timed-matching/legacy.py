import json, subprocess, sys
from pathlib import Path
W=Path('/home/gsc/.cache/glassmind/continuous-20261010'); R=W/'timedmatch-run'; S=W/'factmatch-source'
stage=sys.argv[1]; out=R/'evidence'/stage;out.mkdir(exist_ok=True)
binary=R/'baseline/glassmind-facts' if stage=='baseline' else R/'target/release/glassmind-facts'
proof=R/'baseline/continuous_predictions' if stage=='baseline' else R/'target/release/examples/continuous_predictions'
def run(args,inp=None): return subprocess.run(list(map(str,args)),input=inp,capture_output=True,check=True).stdout
rows=(S/'valid.tsv').read_text().splitlines(); questions=''.join(r.split('\t',3)[3]+'\n' for r in rows).encode()
for n in [200,len(rows)]:
    data=b'\n'.join(questions.splitlines()[:n])+b'\n'
    pred=run([proof,S/'relation.json',S/'index.json'],data); (out/f'predictions-{n}.jsonl').write_bytes(pred)
    score=run([binary,'score','--input',S/'valid.tsv','--model',S/'relation.json','--index',S/'index.json','--limit',n]);(out/f'score-{n}.json').write_bytes(score)
    if stage!='baseline':
        assert pred==(R/'evidence/baseline'/f'predictions-{n}.jsonl').read_bytes()
        assert score==(R/'evidence/baseline'/f'score-{n}.json').read_bytes()
cases=[['verify-at','--temporal',W/'timedmatch-source/temporal.json','--date','1980-01-01','Did Q1 work for Q2?'],['verify-at','--temporal',W/'timedmatch-source/temporal.json','--date','invalid','Did Q1 work for Q2?'],['match','Who worked for Q2 and was a citizen of Q3?'],['match','Who was born in Missing and died in Q3?'],['verify','Did Q1 work for Q2?'],['ask','--model',S/'relation.json',rows[0].split('\t',3)[3]]]
replies=[run([binary,c[0],'--index',S/'index.json',*c[1:]]).decode() for c in cases]
(out/'old-api.json').write_text(json.dumps(replies))
if stage!='baseline': assert replies==json.loads((R/'evidence/baseline/old-api.json').read_text())
print(stage,'200/full legacy bytes and native scores',len(rows),'old APIs',len(cases))
