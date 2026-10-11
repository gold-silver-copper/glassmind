#!/usr/bin/env python3
"""Offline TRAIN diagnostic; annotations never enter frozen token inference."""
import argparse, collections, hashlib, json, pathlib
p=argparse.ArgumentParser();p.add_argument('mode',choices=['prepare','report']);p.add_argument('--limit',type=int,default=100);p.add_argument('--root',type=pathlib.Path,required=True);p.add_argument('--run',type=pathlib.Path,required=True);p.add_argument('--baseline',type=pathlib.Path,required=True);a=p.parse_args()
a.run.mkdir(parents=True,exist_ok=True)
ds=sorted(json.loads((a.root/'selected/train.json').read_text()),key=lambda d:hashlib.sha256(d['dialogue_id'].encode()).hexdigest())[:a.limit or None]
def split(d):return 'calibration' if int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0 else 'fit'
if a.mode=='prepare':
 (a.run/'train-inputs.json').write_text(json.dumps([[{k:t[k] for k in ('speaker','utterance')} for t in d['turns']] for d in ds]))
 (a.run/'selection.json').write_text(json.dumps({'selection':f'first{a.limit or len(ds)} SHA256(dialogue_id) ascending; mod5==0 calibration','ids':[d['dialogue_id'] for d in ds],'conversations':dict(collections.Counter(map(split,ds))),'user_turns':sum(t['speaker']=='USER' for d in ds for t in d['turns'])},indent=2)+'\n')
 m=json.loads((a.baseline/'model.json').read_text());m['version']=3;m['contiguous_spans']=True
 (a.run/'model.json').write_text(json.dumps(m,separators=(',',':')))
else:
 traces=json.loads((a.run/'train-traces.json').read_text());events=[];counts=collections.defaultdict(collections.Counter)
 def decode(tokens,contiguous):
  spans=[];last=None
  for i,(_,start,end,label) in enumerate(tokens):
   if ':' not in label:continue
   tag,slot=label.split(':',1)
   if tag=='I' and spans and spans[-1][0]==slot and (not contiguous or last==i-1):spans[-1][2]=end
   else:spans.append([slot,start,end])
   last=i
  out={}
  for slot,start,end in sorted(spans,key=lambda x:x[2]-x[1]):out[slot]=(start,end)
  return out
 for d,trace in zip(ds,traces):
  for r in trace:
   t=d['turns'][r['turn']];g=next((f for f in t['frames'] if f['service']==r['service']),None)
   if g is None:continue
   before=decode(r['tokens'],False);after=decode(r['tokens'],True)
   for slot in before.keys()|after.keys():
    if before.get(slot)==after.get(slot):continue
    val=lambda x:t['utterance'][slice(*x)] if x else None
    b=val(before.get(slot));c=val(after.get(slot));gold=g['state']['slot_values'].get(slot,[])
    status=lambda v:'correct' if v is not None and v.lower() in [x.lower() for x in gold] else 'wrong'
    transition=status(b)+'->'+status(c);counts[split(d)][transition]+=1
    events.append(dict(dialogue_id=d['dialogue_id'],turn_index=r['turn'],service=r['service'],slot=slot,split=split(d),utterance=t['utterance'],tokens=r['tokens'],before=b,after=c,gold=gold,transition=transition))
 report={'counts':counts,'events':events,'caveat':'Gold-active TRAIN frames only, raw span decoder diagnostics, not session scores; frozen classifiers trained on both partitions, not independent model generalization.'}
 (a.run/'train-diagnostic.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(counts))
