from pathlib import Path
import argparse,hashlib,json,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--model',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
HERE=Path(__file__).resolve().parent
for name,sha in json.loads((HERE/'freeze.json').read_text()).items():assert hashlib.sha256((HERE/name).read_bytes()).hexdigest()==sha,name
cases=json.loads((HERE/'cases.json').read_text());baseline=json.loads((HERE/'baseline.json').read_text());a.out.mkdir(exist_ok=False)
def valid(row):
    heads=row['heads'];labels=row['deprels'];n=len(row['forms'])
    assert len(heads)==len(labels)==n and heads.count(0)==1
    assert row['status']=='predicted' and row['structurally_valid']
    for d,h in enumerate(heads,1):
        assert isinstance(h,int) and 0<=h<=n and h!=d
        assert (labels[d-1]=='root')==(h==0)
        seen=set();v=d
        while v:
            assert v not in seen;seen.add(v);v=heads[v-1]
reports=[]
for mode in ['forms','text']:
    output=a.out/(mode+'.jsonl')
    subprocess.run([str(a.binary),'predict','--model',str(a.model),'--input',str(HERE/(mode+'.jsonl')),'--output',str(output)],check=True,timeout=120)
    rows=[json.loads(s) for s in output.read_text().splitlines()];assert len(rows)==len(cases)
    correctness=[]
    for c,row in zip(cases,rows):
        valid(row);assert row['forms']==c['forms'],(c,row)
        correctness.append([row['heads'][e['dependent']-1]==e['head'] and row['deprels'][e['dependent']-1]==e['label'] for e in c['edges']])
        if mode=='text':
            spans=row['spans'];text=c['text'].encode();assert len(spans)==len(c['forms']);end=0
            for form,s in zip(c['forms'],spans):
                assert end<=s['start']<s['end']<=len(text)
                assert text[s['start']:s['end']].decode()==form==s['form'];end=s['end']
    losses=[dict(case=c['name'],edge=e) for c,old,new in zip(cases,baseline['correctness'],correctness) for e,b,v in zip(c['edges'],old,new) if b and not v]
    reports.append(dict(mode=mode,cases=len(cases),edges=sum(map(len,correctness)),right=sum(sum(x) for x in correctness),complete=sum(all(x) for x in correctness),old_correct_lost=losses,gate=not losses,correctness=correctness))
report=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(a.model.read_bytes()).hexdigest(),baseline_right=baseline['right'],modes=reports,accepted_core_gate=all(x['gate'] for x in reports))
(a.out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='modes'}))
if not report['accepted_core_gate']:raise SystemExit(1)
