"""Parent target oracle from archived TRAIN gold graphs; no worker access."""
from pathlib import Path
from fractions import Fraction
from collections import Counter
import hashlib,json

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
SOURCE=ROOT/'groundingaudit-source'
ARCHIVE=ROOT/'groundingaudit-run/frozen-candidate'
CONSTANTS=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return [json.loads(x) for x in p.read_text().splitlines()]
def action(a):
    k,i=a['kind'],a['index']
    if not isinstance(i,int) or i<0:raise ValueError('invalid index')
    if k=='input' and i<16:return i
    if k=='constant' and i<20:return 16+i
    if k=='step' and i<6:return 36+i
    raise ValueError('invalid argument')
def canonical(op,a,b):
    if op not in ['add','subtract','multiply','divide']:raise ValueError('invalid operation')
    if op in ['add','multiply']:a,b=sorted([a,b])
    return (op,a,b)
def pair_sets(program,inputs):
    steps=program['steps']
    if not 1<=len(steps)<=6 or not 0<=inputs<=16:raise ValueError('limits')
    nodes={i:('input',str(i)) for i in range(inputs)}
    nodes.update({16+i:('constant',str(Fraction(v))) for i,v in enumerate(CONSTANTS)})
    output=[]
    for depth,s in enumerate(steps):
        left,right=action(s['left']),action(s['right'])
        # Missing inputs and forward references are rejected before enumeration.
        gold=canonical(s['op'],nodes[left],nodes[right])
        pairs=[[l,r] for l in sorted(nodes) for r in sorted(nodes) if canonical(s['op'],nodes[l],nodes[r])==gold]
        assert [left,right] in pairs
        output.append(dict(depth=depth,op=s['op'],gold_left=left,gold_right=right,pairs=pairs))
        nodes[36+depth]=gold
    return output

def main():
    out=HERE/'corpus';out.mkdir(exist_ok=False)
    m=json.loads((ARCHIVE/'manifest.json').read_text())['files']
    for name in ['audit/clean.jsonl','audit/membership.jsonl','audit/diagnostics.jsonl']:
        p=ARCHIVE/name;assert sha(p)==m[name]['sha256'] and p.stat().st_size==m[name]['bytes']
    for e in json.loads((SOURCE/'manifest.json').read_text())['files']:
        p=SOURCE/e['file'];assert sha(p)==e['sha256'] and p.stat().st_size==e['bytes']
    clean=read(ARCHIVE/'audit/clean.jsonl');membership=read(ARCHIVE/'audit/membership.jsonl')
    native=read(SOURCE/'train-executions.jsonl');split=json.loads((SOURCE/'split.json').read_text())
    assert len(native)==len(membership)==len(split['group_hashes'])==29837
    assert Counter(c['split'] for c in clean)=={'FIT':4590,'CAL':1182}
    assert [c['index'] for c in clean if c['split']=='FIT']==[m['index'] for m in membership if m['split']=='FIT' and m['clean']]
    assert [c['index'] for c in clean if c['split']=='CAL']==[m['index'] for m in membership if m['split']=='CAL' and m['clean']]
    assert [dict(question=c['question'],program=c['program']) for c in clean if c['split']=='FIT']==read(SOURCE/'fit-input.jsonl')
    counts={s:Counter() for s in ['FIT','CAL']}
    with (out/'targets.jsonl').open('w') as f:
        for c in clean:
            execution=native[c['index']];assert execution['status']=='proposed' and execution['program']==c['program']
            sets=pair_sets(c['program'],len(execution['numbers']))
            row=dict(index=c['index'],split=c['split'],input_count=len(execution['numbers']),steps=sets)
            f.write(json.dumps(row,separators=(',',':'))+'\n')
            count=counts[c['split']];count['rows']+=1
            for s in sets:
                count['steps']+=1;count['target_pairs']+=len(s['pairs']);count['multiple_pairs']+=len(s['pairs'])>1
                count['multiple_left_targets']+=len({p[0] for p in s['pairs']})>1
                count['multiple_right_given_gold_left']+=sum(l==s['gold_left'] for l,r in s['pairs'])>1
    assert counts['CAL']['steps']==4138
    report=dict(scope='TRAIN-only formal target oracle, no worker source/results used',counts={s:dict(c) for s,c in counts.items()},targets_sha256=sha(out/'targets.jsonl'),clean_sha256=sha(ARCHIVE/'audit/clean.jsonl'),script_sha256=sha(Path(__file__)))
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
