#!/usr/bin/env python3
"""Fixed first10 FIT parity for the frozen BIO decoder, independently in Python."""
import json,pathlib,numpy as np
A=pathlib.Path(__file__).resolve().parent/'artifacts'
def read(p):return json.loads(p.read_text())
m=read(A/'model.json')['boundary'];w=np.array(m['w'],dtype=np.float32)
ds=[d for d in read(A/'dataset.json')['dialogues'] if not d['cal']][:10];ids={d['id'] for d in ds};expected={}
for line in open(A/'source-spans.jsonl'):
 z=json.loads(line);r=z['row']
 if r['id'] in ids:expected.setdefault((r['id'],r['turn']),[]).append(r['raw'])
def fnv(s):
 h=2166136261
 for b in s.encode():h=((h^b)*16777619)&0xffffffff
 return h
def tokens(s):
 out=[];start=None;byte=0
 for ch in s:
  n=len(ch.encode())
  if ch.isalnum():
   if start is None:start=byte
  else:
   if start is not None:out.append((start,byte,s.encode()[start:byte].decode()));start=None
   if not ch.isspace():out.append((byte,byte+n,ch))
  byte+=n
 if start is not None:out.append((start,byte,s.encode()[start:byte].decode()))
 return out
def shape(s):return ''.join('A' if c.isupper() else 'a' if c.islower() else '0' if c.isnumeric() else c for c in s)
count=spans=0
for d in ds:
 for turn,t in enumerate(d['turns']):
  if t['input']['speaker']!='SYSTEM':continue
  text=t['input']['utterance'];ts=tokens(text);ys=[];margins=[];prev=0;out=[];count+=1
  if len(text.encode())<=4096 and len(ts)<=128:
   for i,(_,_,word) in enumerate(ts):
    f=['bias']
    for offset in range(-2,3):
     if 0<=i+offset<len(ts):
      word2=ts[i+offset][2];f.extend([f'w{offset}={word2.lower()}',f's{offset}={shape(word2)}'])
    for n in range(1,4):f.extend([f'pre{n}={word.lower()[:n]}',f'suf{n}={word.lower()[::-1][:n]}'])
    fs=sorted({fnv(x) for x in f})+[fnv(f'prev={prev}')]
    scores=[float(np.cumsum(w[np.array(fs)%65536+l*65536],dtype=np.float32)[-1]) for l in range(3)]
    valid=[0,1] if prev==0 else [0,1,2];l=max(valid,key=lambda l:scores[l]);margin=scores[l]-max(scores[j] for j in valid if j!=l)
    ys.append(l);margins.append(margin);prev=l
   i=0
   while i<len(ts):
    if ys[i]!=1:i+=1;continue
    first=i;i+=1
    while i<len(ts) and ys[i]==2:i+=1
    if i-first<=16:out.append({'start':ts[first][0],'end':ts[i-1][1],'boundary_margin':min(margins[first:i])})
  exp=expected.get((d['id'],turn),[]);assert len(out)==len(exp)
  for a,b in zip(out,exp):
   assert (a['start'],a['end'])==(b['start'],b['end']) and abs(a['boundary_margin']-b['boundary_margin'])<0.002
  spans+=len(out)
r={'fixed_first_FIT_dialogues':[d['id'] for d in ds],'SYSTEM_turns':count,'raw_spans':spans,'independent_boundary_decode_matches':True}
(A/'boundary-parity.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
