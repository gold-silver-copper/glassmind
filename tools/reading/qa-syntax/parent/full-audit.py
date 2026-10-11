# SPDX-License-Identifier: MIT
"""Independent retained-output audit; no worker scorer or runtime imports."""
from pathlib import Path
import argparse,collections,hashlib,json,re,string
p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--run',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
def norm(s):
 return ' '.join(re.sub(r'\b(a|an|the)\b',' ',''.join(c for c in s.lower() if c not in string.punctuation)).split())
def f1(g,p):
 g,p=norm(g).split(),norm(p).split()
 if not g or not p:return float(g==p)
 n=sum((collections.Counter(g)&collections.Counter(p)).values())
 return 2*n/(len(g)+len(p)) if n else 0.
def lines(path):return [json.loads(x) for x in path.read_text().splitlines()]
def span(row,p):
 raw=''
 if p['proposal']:
  s=p['proposal'];e=p['evidence'];b=row['context'].encode();assert 0<=e['start']<=s['start']<s['end']<=e['end']<=len(b);raw=b[s['start']:s['end']].decode()
 assert p['answer']==(raw if p['status']=='predicted' else '')
 return raw
def score(row,p):
 raw=span(row,p);gold=[x['text'] for x in row['answers'] if norm(x['text'])] or ['']
 exact=any(norm(raw)==norm(g) for g in gold)
 selected='refused' if p['status']!='predicted' else 'right' if raw and any(norm(g) and norm(raw)==norm(g) for g in gold) else 'wrong'
 return exact,max(f1(g,raw) for g in gold),selected
source=json.loads((a.source/'dev-v1.1.json').read_text())
rows=[dict(id=q['id'],context=p['context'],question=q['question'],answers=q['answers']) for d in source['data'] for p in d['paragraphs'] for q in p['qas']]
rows.sort(key=lambda x:hashlib.sha256(x['id'].encode()).hexdigest());assert len(rows)==len({r['id'] for r in rows})==10570
old=lines(a.run/'baseline/full.candidate.outputs.jsonl');new=lines(a.run/'experiment/full.outputs.jsonl');inputs=lines(a.run/'experiment/full.input.jsonl');assert len(old)==len(new)==len(inputs)==len(rows)
counts=[collections.Counter(),collections.Counter()];transitions={k:collections.Counter() for k in ('raw','selected')}
for row,b,c,inp in zip(rows,old,new,inputs):
 assert inp=={k:row[k] for k in ('context','question')};assert c['semantic_status']=='unverified'
 before,after=score(row,b),score(row,c)
 for total,(em,f,state) in zip(counts,(before,after)):total.update({'raw_exact':em,'f1_sum':f,state:1})
 transitions['raw'][('right' if before[0] else 'wrong')+'->'+('right' if after[0] else 'wrong')]+=1
 transitions['selected'][before[2]+'->'+after[2]]+=1
for c in counts:c['raw_f1_percent']=100*c.pop('f1_sum')/len(rows)
assert counts[0]['raw_exact']==2532 and counts[1]['raw_exact']==3057
assert (counts[0]['right'],counts[0]['wrong'],counts[0]['refused'])==(815,791,8964)
assert (counts[1]['right'],counts[1]['wrong'],counts[1]['refused'])==(943,647,8980)
cal=json.loads((a.run/'experiment/cal.labels.json').read_text());pred=lines(a.run/'experiment/cal.outputs.jsonl');assert len(cal)==len(pred)==1000
ranked=[]
for r,p in zip(cal,pred):
 raw=span(r,p)
 if p['proposal'] and p['reason']!='no_lexical_support':ranked.append((p['margin'],int(any(norm(g['text'])==norm(raw) for g in r['answers']))))
ranked.sort(reverse=True);right=0;selected=None
for i,(margin,correct) in enumerate(ranked,1):
 right+=correct
 if i<len(ranked) and ranked[i][0]==margin:continue
 if i>=30 and right/i>=.5:selected=dict(threshold=margin,accepted=i,right=right,wrong=i-right)
model=json.loads((a.run/'experiment/model.json').read_text());assert selected['threshold']==model['ranker']['threshold'];assert selected['accepted']==166 and selected['right']==83
result=dict(total=len(rows),baseline=dict(counts[0]),candidate=dict(counts[1]),transitions={k:dict(v) for k,v in transitions.items()},calibration=selected,scope='Independent recomputation on frozen DEV outputs; does not override independent negation rejection.')
a.out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
