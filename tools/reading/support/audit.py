#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Post-freeze diagnostics only: never selects a threshold or modifies a model."""
import collections,json
from experiment import RUN,ROOT,qa,read,score_tree,infer
model=json.loads((RUN/'support-model.json').read_text())
ps=read(RUN/'cal.features.jsonl');rs=json.loads((RUN/'cal.labels.json').read_text())
selected=[(r,p) for r,p in zip(rs,ps) if p['prediction']['status']=='predicted']
labels=[max(qa.SCORE['compute_exact'](a['text'],p['prediction']['answer']) for a in r['answers']) for r,p in selected]
scores=[model['bias']+sum(score_tree(t,p['features']) for t in model['trees']) for r,p in selected]
result={}
for name,ss in [('support',scores),('margin',[p['prediction']['margin'] for r,p in selected])]:
 ranked=sorted(zip(ss,labels),reverse=True);right=0;frontier=[]
 for i,(score,y) in enumerate(ranked):
  right+=y
  if i+1<len(ranked) and ranked[i+1][0]==score:continue
  if i+1>=30:frontier.append(dict(accepted=i+1,right=right,precision=right/(i+1)))
 result[name]={'best_observed_precision_with_at_least30':max(frontier,key=lambda x:x['precision']), 'top30_right':sum(y for s,y in ranked[:30]),'frontier':frontier}
names=json.loads((ROOT/'tools/reading/support/features.json').read_text());usage=collections.Counter()
def walk(t):
 if 'value' not in t:usage[names[t['feature']]]+=1;walk(t['left']);walk(t['right'])
for t in model['trees']:walk(t)
result['split_feature_counts']=dict(usage.most_common());result['warning']='Post-freeze diagnostic only, no alternative threshold selected or evaluated.'
qa.write(RUN/'diagnostic.json',result)
# Expose unthresholded scores on the same predeclared probes. No acceptance claims.
ps=infer('probes',features=True);cases=json.loads((RUN/'probes.json').read_text())
for c,p in zip(cases,ps):
 c['classifier_score_not_probability']=model['bias']+sum(score_tree(t,p['features']) for t in model['trees'])
 c['features']=dict(zip(names,p['features']))
qa.write(RUN/'probes.json',cases)
print(json.dumps({k:{a:b for a,b in v.items() if a!='frontier'} if isinstance(v,dict) else v for k,v in result.items()},indent=2))
