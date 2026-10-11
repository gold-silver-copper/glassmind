#!/usr/bin/env python3
"""Measure gold-prior versus frozen-predicted-prior mismatch on TRAIN only."""
import argparse,collections,json,pathlib,subprocess,re
p=argparse.ArgumentParser()
for name in ('train','schema','binary','baseline','relations','output'):
    p.add_argument('--'+name,required=True,type=pathlib.Path)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
ds=json.loads(a.train.read_text())
inp=a.output/'inputs.json';pred=a.output/'predictions.json'
inp.write_text(json.dumps([[{k:t[k] for k in ('speaker','utterance')} for t in d['turns']] for d in ds]))
subprocess.run([str(a.binary),'predict','--input',str(inp),'--schema',str(a.schema),'--model',str(a.baseline),'--output',str(pred)],check=True)
relations=json.loads(a.relations.read_text())['relations'];counts=collections.Counter();examples=[]
for d,ps in zip(ds,json.loads(pred.read_text())):
    prior={}
    for p in ps:
        i=p['turn_index'];gold={f['service']:f['state']['slot_values'] for f in d['turns'][i]['frames'] if 'state' in f}
        current={f['service']:f['state']['slot_values'] for f in p['frames']}
        for r in relations:
            target=r['target_service'];slot=r['target_slot'];source=r['source_service'];ss=r['source_slot']
            if target not in p['active_services'] or slot in current[target]:continue
            values=prior.get(source,{}).get(ss,[])
            if len(values)!=1 or len(values[0].split())>12:continue
            if not any(re.search(r'(?<!\w)'+re.escape(values[0])+r'(?!\w)',t['utterance'],re.I) for t in d['turns'][:i]):continue
            if target not in gold:counts['no_target_gold_frame']+=1;continue
            right=values[0] in gold[target].get(slot,[])
            counts['correct_opportunity' if right else 'incorrect_opportunity']+=1
            if not right and len(examples)<12:
                examples.append(dict(source=source,source_slot=ss,target=target,target_slot=slot,value=values[0],gold=gold[target].get(slot,[]),system=d['turns'][i-1]['utterance'] if i else '',user=d['turns'][i]['utterance']))
        prior=current
(a.output/'report.json').write_text(json.dumps(dict(split='train only',counts=dict(counts),incorrect_examples=examples),indent=2)+'\n')
print(json.dumps(counts,indent=2))
