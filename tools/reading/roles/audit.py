#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Verify frozen artifacts and summarize transitions; never retrain or tune."""
import importlib.util,json,os,pathlib,subprocess,collections,hashlib
here=pathlib.Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('experiment',here/'experiment.py');e=importlib.util.module_from_spec(spec);spec.loader.exec_module(e)
run=e.RUN; base=e.BASE
freeze=e.read(here/'reports/freeze.json');e.frozen(run/'reading-qa')
full=e.read(here/'reports/full.json'); assert not full['passes_frozen_gate']
rows=e.read(run/'full.transitions.json');assert len(rows)==10570 and len({r['id'] for r in rows})==10570
summary={}
for kind in ['raw','selected']:
 rs=[r[kind] for r in rows];counts=collections.Counter(r['transition'] for r in rs)
 assert dict(counts)==full['transitions'][kind]
 summary[kind]=dict(matrix=dict(counts),answers_changed=sum(r['changed'] for r in rs),old_correct_lost=sum(r['transition'].startswith('right->') and r['transition']!='right->right' for r in rs),new_correct=sum(r['transition'].endswith('->right') and r['transition']!='right->right' for r in rs),new_wrong_from_right_or_refused=sum(r['transition'] in ['right->wrong','refused->wrong'] for r in rs),changed_wrong_to_wrong=sum(r['transition']=='wrong->wrong' and r['changed'] for r in rs))
# Recompute the exact CAL prefix gate, preserving all ties and the original rule.
cal=e.read(base/'cal.labels.json'); ps=e.predictions(run/'cal.outputs.jsonl');ranked=[]
for r,p in zip(cal,ps):
 if p['proposal'] and p['reason']!='no_lexical_support':
  ranked.append((p['margin'],max(e.qa.SCORE['compute_exact'](a['text'],e.qa.proposal(r,p)) for a in r['answers'])))
ranked.sort(reverse=True);right=0;front=[]
for i,(margin,correct) in enumerate(ranked):
 right+=correct
 if i+1<len(ranked) and ranked[i+1][0]==margin: continue
 front.append(dict(threshold=margin,accepted=i+1,right=right,precision=right/(i+1)))
selected=[r for r in front if r['accepted']>=30 and r['precision']>=.5][-1]
assert selected['threshold']==e.read(run/'model.json')['threshold'];assert selected['accepted']==86 and selected['right']==43
e.write(run/'cal.frontier.json',front)
for p in ['crates/reading/src/qa.rs','crates/reading/src/bin/reading-qa.rs','crates/reading/src/probability.rs','tools/reading/qa/run.py']:
 assert pathlib.Path(p).read_bytes()==subprocess.check_output(['git','show','HEAD:'+p]),p
assert e.sha(base/'model.json')=='b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5'
assert (run/'v1.replay.outputs.jsonl').read_bytes()==(base/'full.candidate.outputs.jsonl').read_bytes()
subprocess.run(['git','apply','--check',str(here/'rejected.patch')],check=True)
validation=dict(result='REJECTED; runtime restored',transitions=summary,calibration_recomputed=selected,legacy_byte_identical_rows=10570,original_fit_cal_byte_equality=True,qa_tests_passed=8,byte_probability_tests_passed=2,release_build='passed',fmt='passed',clippy='failed:14 experimental indexing/slicing violations; exact evaluated patch retained without post-dev fixes',probability_source_and_scorer_unchanged=True,main_cli_integration='not run: rejected',candidate_full_runs=1,training_runs=1,dev_retuning=False,outer_memory_max=8589934592,outer_memory_swap_max=0)
e.write(here/'reports/validation.json',validation);print(json.dumps(validation,indent=2))
