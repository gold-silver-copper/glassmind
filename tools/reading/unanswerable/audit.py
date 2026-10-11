#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Independent saved-output audit; no inference, training or scorer modification."""
import collections,hashlib,json,subprocess,os,importlib.util
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];WORK=ROOT.parent
RUN=WORK/'unanswerable-run/experiment';BASE=WORK/'unanswerable-run/baseline'
def get(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,x):p.write_text(json.dumps(x,indent=2)+'\n')
selection=get(HERE/'reports/selection.json');source=get(ROOT/'tools/reading/qa2/source.json')
train=get(WORK/'readingqa2-source/train-v2.0.json')['data'];dev=get(WORK/'readingqa2-source/dev-v2.0.json')['data']
old=get(ROOT/'tools/reading/qa/reports/selection.json');ex=set(old['fit']['article_hashes']+old['cal']['article_hashes'])
v1=get(WORK/'readingqa-source/train-v1.1.json')['data']
excluded={a['title'] for a in v1 if hashlib.sha256('\n'.join(p['context'] for p in a['paragraphs']).encode()).hexdigest() in ex}
assert len(excluded)==80 and excluded==set(selection['excluded_titles'])
ordered=sorted([a for a in train if a['title'] not in excluded],key=lambda a:hashlib.sha256(a['title'].encode()).hexdigest())
groups={'fit':ordered[:96],'cal':ordered[96:128],'dev':dev}
for kind in ('title','context','id'):
 def values(aa):
  if kind=='title':return {a['title'] for a in aa}
  if kind=='context':return {p['context'] for a in aa for p in a['paragraphs']}
  return {q['id'] for a in aa for p in a['paragraphs'] for q in p['qas']}
 for a,b in [('fit','cal'),('fit','dev'),('cal','dev')]:assert not values(groups[a])&values(groups[b]),(kind,a,b)
for name,cap in [('fit',8000),('cal',2000)]:
 ids=sorted([q['id'] for a in groups[name] for p in a['paragraphs'] for q in p['qas']],key=lambda x:hashlib.sha256(x.encode()).hexdigest())[:cap]
 assert ids==selection['selection'][name]['ids']
 inputs=[json.loads(s) for s in (RUN/(name+'.input.jsonl')).read_text().splitlines()]
 assert all(set(r)=={'context','question'} for r in inputs)
 assert len(inputs)==cap
 assert selection['selection'][name]['titles']==[a['title'] for a in groups[name]]
assert sha(BASE/'model.json')=='b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5'
assert sha(BASE/'full.outputs.jsonl')==sha(RUN/'v1-replay.output.jsonl')=='6dd02aa2757011bec9971e47aef04e08260426f089eea004396ac13f0f9666a4'
assert len((RUN/'v1-replay.output.jsonl').read_text().splitlines())==11873
front=get(RUN/'cal-frontier.json');cal=get(HERE/'reports/calibration.json')
# Reconstruct every CAL tie group from first-run weights and official exactness.
os.environ['SQUAD_SOURCE']=str(WORK/'readingqa2-source')
spec=importlib.util.spec_from_file_location('qa_official',ROOT/'tools/reading/qa/run.py')
qa=importlib.util.module_from_spec(spec);spec.loader.exec_module(qa)
weights=get(RUN/'first-weights.json')['weights'];labels=get(RUN/'cal.labels.json')
predictions=[json.loads(s) for s in (RUN/'cal.output.jsonl').read_text().splitlines()]
ranked=[]
for row,out in zip(labels,predictions,strict=True):
 raw=qa.proposal(row,out['prediction']);gold=[a['text'] for a in row['answers'] if qa.SCORE['normalize_answer'](a['text'])]
 exact=int(not row['impossible'] and bool(qa.SCORE['normalize_answer'](raw)) and any(qa.SCORE['compute_exact'](g,raw) for g in gold))
 if out['features']:ranked.append((sum(weights[i]*v for i,v in out['features']),exact,int(row['impossible']),row['id']))
ranked.sort(reverse=True);right=wrongnull=0;independent=[]
for i,(score,exact,null,_) in enumerate(ranked):
 right+=exact;wrongnull+=null
 if i+1<len(ranked) and ranked[i+1][0]==score:continue
 independent.append(dict(threshold=score,accepted=i+1,correct=right,wrong=i+1-right,precision=right/(i+1),strict_correct=right+606-wrongnull,expected_refusals=606-wrongnull))
assert front==independent
qualifiers=[r for r in front if r['precision']>=.6 and r['correct']>=100 and r['strict_correct']>cal['always_refuse']]
assert not qualifiers and not cal['gate_pass']
assert not any((RUN/(n+'.output.jsonl')).exists() for n in ('sample','full'))
assert not (RUN/'model.json').exists()
baseqa=subprocess.check_output(['git','show','HEAD:crates/reading/src/qa.rs'])
assert (ROOT/'crates/reading/src/qa.rs').read_bytes()==baseqa # run after restoration
assert subprocess.check_output(['git','diff','--','crates/reading']).strip()==b''
r=subprocess.run(['git','apply','--check',str(HERE/'rejected.patch')],capture_output=True);assert r.returncode==0,r.stderr
max_at60=max([r['correct'] for r in front if r['precision']>=.6],default=0)
write(HERE/'reports/validation.json',dict(status='REJECTED_CAL',tests_passed=8,clippy='passed --lib --bin reading-qa -D warnings',independently_recomputed_cal_frontier=True,default_v1_full11873_replay_byte_identical=True,default_prediction_sha256=sha(RUN/'v1-replay.output.jsonl'),original_v1_model_sha256=sha(BASE/'model.json'),runtime_restored_byte_exact=True,rejected_patch_applies=True,all_selected_article_title_context_id_overlaps=0,removed_train_contexts=0,max_correct_at_precision60=max_at60,first_weights_sha256=sha(RUN/'first-weights.json'),candidate_dev_run=False,source=source,old_squad1_outcomes='Preserved by byte-exact original runtime restoration and unchanged default proposal code; no v1 DEV data read or replayed.',limitations=['No passing CAL threshold; no candidate DEV scores','Hashed lexical alignment is not entailment','No main CLI or workspace-wide cross-track gate run']))
print('Audit passed; max correct with >=60% CAL precision:',max_at60)
