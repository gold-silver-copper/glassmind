"""Complete DEV transition ledger; unchanged independent audit supplies scores."""
import collections, importlib.util, json, pathlib, sys
spec=importlib.util.spec_from_file_location('audit', 'tools/reading/syntax/parent/audit.py')
audit=importlib.util.module_from_spec(spec); spec.loader.exec_module(audit)
source, baseline, out=map(pathlib.Path,sys.argv[1:])
def read(p): return [json.loads(x) for x in p.read_text().splitlines()]
gold=audit.corpus(source/'en_ewt-ud-dev.conllu')
train={tuple(x[0] for x in s) for s in audit.corpus(source/'en_ewt-ud-train.conllu')}
old=read(baseline/'forms-predictions.jsonl'); new=read(out/'full-predictions.jsonl')
assert len(old)==len(new)==len(gold)==2001
counts=collections.defaultdict(collections.Counter)
with (out/'edge-transitions.jsonl').open('w') as ef, (out/'sentence-transitions.jsonl').open('w') as sf:
 for si,(g,b,c) in enumerate(zip(gold,old,new)):
  duplicate=tuple(x[0] for x in g) in train
  groups=['all','duplicate_forms' if duplicate else 'nonduplicate_forms']
  assert c['status']=='predicted' and audit.is_tree(c['heads'],c['deprels'])
  sr={'sentence':si,'duplicate':duplicate}
  for metric in ['uas','las']:
   before=[]; after=[]
   for i,(_,_,h,l) in enumerate(g):
    bo=b['heads'][i]==h and (metric=='uas' or b['deprels'][i]==l)
    ca=c['heads'][i]==h and (metric=='uas' or c['deprels'][i]==l)
    before.append(bo);after.append(ca)
    state=f'{int(bo)}->{int(ca)}'
    for group in groups: counts[group][metric+'_edge_'+state]+=1
    ef.write(json.dumps(dict(sentence=si,token=i+1,duplicate=duplicate,metric=metric,before=bo,after=ca,gold=[h,l],old=[b['heads'][i],b['deprels'][i]],new=[c['heads'][i],c['deprels'][i]]))+'\n')
   state=f'{int(all(before))}->{int(all(after))}'
   for group in groups: counts[group][metric+'_sentence_'+state]+=1
   sr[metric]={'before':all(before),'after':all(after),'before_edges':sum(before),'after_edges':sum(after)}
  sf.write(json.dumps(sr)+'\n')
score=json.loads((out/'audit/score.json').read_text())['groups']['all']
r={'groups':dict(counts),'gates':{'las':score['las_correct']>=19673,'uas':score['uas_correct']>=20540,'all_valid_emitted':score['emitted']==2001},'parent_core_retention':'outstanding; parent inputs not accessed'}
(out/'transitions.json').write_text(json.dumps(r,indent=2)+'\n'); print(json.dumps(r,indent=2))
