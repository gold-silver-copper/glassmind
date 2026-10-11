from pathlib import Path
import collections,hashlib,json,re
HERE=Path(__file__).resolve().parent;WORK=HERE.parents[1];REPO=Path('/home/gsc/code/glassmind')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
train=Path('/home/gsc/.cache/glassmind/data/simplequestions/train.tsv');assert sha(train)=='25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b'
art=WORK/'linkquality-run/artifacts';wanted=[art/'train.questions',art/'new999-train-pairs.jsonl']
manifest=json.loads((REPO/'tools/facts/linkquality/artifacts.json').read_text())['artifacts'];by={x['path']:x for x in manifest}
for p in wanted:
 ident=by[str(p)];assert sha(p)==ident['sha256'] and p.stat().st_size==ident['bytes']
index=WORK/'trainindex-run/tmp/trainindex/bundle1000/index.json';assert sha(index)=='142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791'
selection=WORK/'trainindex-run/tmp/trainindex/selection.json';assert sha(selection)=='77208912a8fbf86efc3aafa707495c6c5f1ecde2f07390d9a37d2ece637f40af'
rows=[s.split('\t',3) for s in train.read_text().splitlines()];qs=wanted[0].read_text().splitlines();pred=[json.loads(s)['baseline'] for s in wanted[1].read_text().splitlines()];assert len(rows)==len(qs)==len(pred)==19481
assert qs==[r[3] for r in rows]
entities=json.loads(index.read_text())['entities'];chosen={r['qid'] for r in json.loads(selection.read_text())['entities']};assert len(chosen)==1000 and set(entities)<chosen
end=len(rows)*9//10;eligible=[i for i in range(end) if i%10!=0]
normalize=lambda s:' '.join(''.join(c.lower() if c.isalnum() else ' ' for c in s).split())
def aliases(e,q):
 text=' '+normalize(q)+' '
 return sorted({normalize(a) for a in e['aliases'] if len(normalize(a))>=2 and ' '+normalize(a)+' ' in text})
counts=collections.defaultdict(collections.Counter);idroles=collections.defaultdict(collections.Counter);samples=collections.defaultdict(list)
for i in eligible:
 anchor,relation,answer,q=rows[i];direction=relation[0];b=pred[i];c=counts[direction];c['rows']+=1;idroles[anchor][direction]+=1
 present=anchor in entities;c['anchor_in_index']+=present;c['linked_correct_anchor']+=b['entity']==anchor
 has_alias=bool(aliases(entities[anchor],q)) if present else False;c['anchor_alias_in_question']+=has_alias
 if b['refusal'] is not None:
  c['refused']+=1;c['refusal:'+b['refusal']]+=1
 elif b['answers']:
  c['answered']+=1;c['answer_match']+=answer in b['answers'];good=b['entity']==anchor and b['relation']==relation and answer in b['answers'];c['strict_joint']+=good
  if b['entity']!=anchor:
   category='wrong_anchor:'+('present_alias' if has_alias else 'present_no_alias' if present else 'absent')
   c[category]+=1
   if len(samples[direction+':'+category])<5:samples[direction+':'+category].append(dict(index=i,question=q,gold_anchor=anchor,predicted=b,selected_aliases=aliases(entities[b['entity']],q)))
  elif not good:c['correct_anchor_wrong_relation_or_answer']+=1
  if good and len(samples[direction+':correct'])<5:samples[direction+':correct'].append(dict(index=i,question=q,gold_anchor=anchor,predicted=b))
 else:raise AssertionError(b)
selection_roles=collections.Counter()
for q in chosen:
 tags=tuple(sorted(idroles[q]));selection_roles['+'.join(tags) or 'none']+=1
all_roles=collections.Counter('+'.join(sorted(c)) for c in idroles.values())
report=dict(scope='selection FIT only: prefix90%, excluding i%10==0; in-sample diagnostic',selection_rows=len(eligible),counts=dict(counts),eligible_unique_anchors=len(idroles),eligible_anchor_directions=dict(all_roles),selected_anchor_directions=dict(selection_roles),samples=dict(samples),hashes={str(p):sha(p) for p in [train,*wanted,index,selection]})
(HERE/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['samples','hashes']},indent=2))
