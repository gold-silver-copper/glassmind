#!/usr/bin/env python3
"""Compare retained typed traces, with refusals distinct from wrong actions."""
import argparse, json, pathlib
p=argparse.ArgumentParser(); p.add_argument('baseline',type=pathlib.Path); p.add_argument('candidate',type=pathlib.Path); p.add_argument('output',type=pathlib.Path); a=p.parse_args()
b=json.loads(a.baseline.read_text()); c=json.loads(a.candidate.read_text())
assert [(x['expected'],x['value'],x['text']) for x in b]==[(x['expected'],x['value'],x['text']) for x in c]
def score(rows):
    out=dict(total=len(rows),rename_correct=0,rename_refused=0,rename_wrong=0,negative_safe=0,negative_mutating=0,repair_correct=0,repair_refused=0)
    for r in rows:
        ms=r['meanings']
        if r['expected']=='rename':
            bits=r['value'].split(':')
            if len(bits)==2: kind=None; old,new=bits; owner=None
            else: kind,old,new,*owners=bits; owner=owners[0] if owners else None
            def right(m):
                oldform='Called' if owner else 'Named'
                good=m.startswith('Command(Rename {') and f'{oldform}(Name({json.dumps(old,ensure_ascii=False)}))' in m and f'name: Some(Name({json.dumps(new,ensure_ascii=False)}))' in m
                if kind: good &= ('kind: None' if kind=='_' else 'kind: Some('+kind.capitalize()+')') in m
                if owner: good &= f'other: Named(Name({json.dumps(owner,ensure_ascii=False)}))' in m
                return good
            out['rename_correct' if ms and all(right(m) for m in ms) else 'rename_wrong' if any('Command(' in m for m in ms) else 'rename_refused']+=1
        elif r['expected']=='refuse':
            out['negative_mutating' if any('Command(' in m for m in ms) else 'negative_safe']+=1
        else:
            out['repair_correct' if ms and all(m=='Command(Repair(Test))' for m in ms) else 'repair_refused']+=1
    return out
out=dict(baseline=score(b),candidate=score(c),ask_do_equal=sum(r.get('same_meanings',False) for r in c),sources_unchanged=sum(r.get('source_unchanged',False) for r in c),changed=[dict(text=y['text'],baseline=x['meanings'],candidate=y['meanings']) for x,y in zip(b,c) if x['meanings']!=y['meanings']])
a.output.write_text(json.dumps(out,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:v for k,v in out.items() if k!='changed'},indent=2))
