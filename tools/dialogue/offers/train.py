#!/usr/bin/env python3
"""Fit auditable boundary rules using TRAIN only; no slot-span/action labels."""
import argparse, collections, hashlib, json, pathlib, re
TOKEN = re.compile(r"[^\W_]+(?:['’-][^\W_]+)*|[^\s]", re.UNICODE)
NEG = {'no','not','never','instead','another','different',"don't",'or'}
def tokens(s): return [(m.group().lower(),m.start(),m.end()) for m in TOKEN.finditer(s)]
def safe(s): return not (set(t[0] for t in tokens(s)) & NEG)
CONNECTORS = {'of','the','in','and','on','at','de','des','le','la','to','for'}
def shape(s):
 ts=tokens(s)
 if not ts or not s[ts[0][1]].isalnum() or not (s[ts[0][1]].isupper() or s[ts[0][1]].isdigit()):return False
 return all(s[a].isupper() or s[a].isdigit() or w in CONNECTORS or w in {"'",'’','-','&','.','(',')'} for w,a,b in ts)
def extract(s,left,right):
 ts=tokens(s)
 if len(ts)>80 or not safe(s): return []
 words=['<bos>']+[t[0] for t in ts]+['<eos>'];out=[]
 for a in range(len(ts)):
  if words[max(0,a+1-len(left)):a+1]!=list(left):continue
  for b in range(a+1,min(len(ts),a+16)+1):
   if words[b+1:b+1+len(right)]!=list(right):continue
   v=s[ts[a][1]:ts[b-1][2]]
   if len(v.split())<=12 and shape(v):out.append((v,ts[a][1],ts[b-1][2]))
 return out

def main():
 p=argparse.ArgumentParser()
 for x in ['train','schema','baseline','output']:p.add_argument('--'+x,required=True)
 a=p.parse_args();ds=json.load(open(a.train));schemas=json.load(open(a.schema))
 targets=[]
 for s in schemas:
  inputs={k for it in s['intents'] for k in it['required_slots']+list(it['optional_slots'])}
  targets += [(s['service_name'],q['name']) for q in s['slots'] if not q['is_categorical'] and q['name'] in inputs]
 rows={t:[[],[]] for t in targets};rules=collections.defaultdict(set);values=collections.defaultdict(set)
 split_counts=[0,0]
 for d in ds:
  fold=int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0;split_counts[fold]+=1
  for i,t in enumerate(d['turns']):
   if t['speaker']!='USER' or i==0 or d['turns'][i-1]['speaker']!='SYSTEM':continue
   system=d['turns'][i-1]['utterance'];user=t['utterance']
   if not safe(system) or not safe(user):continue
   ts=tokens(system)
   if len(ts)>80:continue
   words=['<bos>']+[w[0] for w in ts]+['<eos>']
   frames={f['service']:f.get('state',{}).get('slot_values',{}) for f in t['frames']}
   for target in targets:
    service,slot=target
    if service not in frames:continue
    gold=[v.lower() for v in frames[service].get(slot,[])]
    rows[target][fold].append((system,gold,(d['dialogue_id'],i-1)))
    if fold:continue
    for start in range(len(ts)):
     for end in range(start+1,min(len(ts),start+16)+1):
      value=system[ts[start][1]:ts[end-1][2]]
      if value.lower() not in gold or len(value.split())>12:continue
      for l in range(1,4):
       for r in range(1,4):
        left=tuple(words[max(0,start+1-l):start+1]);right=tuple(words[end+1:end+1+r])
        key=target+(left,right)
        rules[key].add((d['dialogue_id'],i-1));values[key].add(value.lower())
 kept=[];diagnostics=[]
 for key,support in sorted(rules.items()):
  if len(support)<5 or len(values[key])<5:continue
  target=key[:2];left,right=key[2:];counts=[]
  for fold in (0,1):
   good=set();bad=set()
   for system,gold,source in rows[target][fold]:
    for v,_,_ in extract(system,left,right):
     (good if v.lower() in gold else bad).add(source)
   counts.append((len(good),len(bad)))
  accepted=counts[0][0]>=5 and counts[0][1]==0 and counts[1][0]>=3 and counts[1][1]==0
  rec={'service':target[0],'slot':target[1],'left':left,'right':right,'fit_correct':counts[0][0],'fit_wrong':counts[0][1],'calibration_correct':counts[1][0],'calibration_wrong':counts[1][1],'distinct_values':len(values[key])}
  diagnostics.append(dict(rec,accepted=accepted))
  if accepted:kept.append(rec)
 model={'version':1,'rules':kept}
 out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=True)
 (out/'offers.json').write_text(json.dumps(model,indent=2)+'\n')
 (out/'training-report.json').write_text(json.dumps({'split_conversations':split_counts,'targets':targets,'proposed_rules':len(rules),'rules':diagnostics,'accepted':len(kept)},indent=2)+'\n')
 base=json.load(open(a.baseline));assert 'offers' not in base;base['offers']=model
 (out/'model.json').write_text(json.dumps(base,separators=(',',':')))
 print(json.dumps({'split':split_counts,'proposed':len(rules),'considered':len(diagnostics),'accepted':len(kept),'by_target':dict(collections.Counter(r['service']+'.'+r['slot'] for r in kept))}))
if __name__=='__main__':main()
