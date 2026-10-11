#!/usr/bin/env python3
"""Audit retained public DEV predictions without inference or training.

Use the unchanged official get_raw_scores function: it discards gold variants
that normalize to empty when other variants exist. The worker transition helper
did not, so three refusals were mislabeled as correct in its transition table.
"""
import collections
import importlib.util
import json
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('qa', HERE.parent / 'qa/run.py')
qa = importlib.util.module_from_spec(spec)
spec.loader.exec_module(qa)
run = Path(os.environ['ROLES_RUN'])
base = Path(os.environ['ROLES_BASE'])


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def exact_scores(rows, texts):
    dataset = [dict(paragraphs=[dict(qas=[dict(id=r['id'], answers=r['answers']) for r in rows])])]
    return qa.SCORE['get_raw_scores'](dataset, dict(zip([r['id'] for r in rows], texts, strict=True)))


rows = sorted(qa.rows(qa.load('dev-v1.1.json')['data']), key=qa.order)
predictions = {'baseline': lines(base/'full.candidate.outputs.jsonl'),
               'candidate': lines(run/'full.outputs.jsonl')}
assert len(rows) == len({r['id'] for r in rows}) == 10570
assert lines(base/'full.input.jsonl') == [dict(context=r['context'], question=r['question']) for r in rows]
full = json.loads((HERE/'reports/full.json').read_text())
scores, states = {}, {}
for name, ps in predictions.items():
    assert len(ps) == len(rows)
    states[name] = {}
    scores[name] = {}
    for kind in ('raw', 'selected'):
        texts = [qa.proposal(r, p) if kind == 'raw' else p['answer'] for r, p in zip(rows, ps, strict=True)]
        exact, f1 = exact_scores(rows, texts)
        states[name][kind] = ['refused' if kind == 'selected' and p['status'] == 'refused'
                              else ('right' if exact[r['id']] else 'wrong')
                              for r, p in zip(rows, ps, strict=True)]
        counts = dict(collections.Counter(states[name][kind]))
        scores[name][kind] = dict(counts=counts, official_exact=sum(exact.values()),
                                  official_f1_percent=sum(f1.values())*100/len(rows))
        if kind == 'selected':
            assert all(counts[k] == full[name][k] for k in ('right', 'wrong', 'refused'))
        else:
            assert abs(sum(f1.values())*100/len(rows)-full[name]['raw']['f1']) < 1e-9
transitions = {kind: dict(collections.Counter(a+'->'+b for a,b in
               zip(states['baseline'][kind], states['candidate'][kind], strict=True)))
               for kind in ('raw', 'selected')}
assert transitions['raw'] == full['transitions']['raw']
corrections = {key: value-full['transitions']['selected'].get(key, 0)
               for key,value in transitions['selected'].items()
               if value != full['transitions']['selected'].get(key, 0)}
assert corrections == {'refused->refused': 3, 'right->right': -3}
assert (run/'v1.replay.outputs.jsonl').read_bytes() == (base/'full.candidate.outputs.jsonl').read_bytes()

cal = json.loads((base/'cal.labels.json').read_text())
ps = lines(run/'cal.outputs.jsonl')
assert len(cal) == len(ps) == 1000
ranked = sorted([(p['margin'], max(qa.SCORE['compute_exact'](a['text'], qa.proposal(r,p))
                                 for a in r['answers']))
                 for r,p in zip(cal,ps,strict=True)
                 if p['proposal'] and p['reason'] != 'no_lexical_support'], reverse=True)
right = 0
selected = None
for i,(margin,correct) in enumerate(ranked):
    right += correct
    if i+1 < len(ranked) and ranked[i+1][0] == margin:
        continue
    if i+1 >= 30 and right/(i+1) >= .5:
        selected = dict(threshold=margin,right=right,accepted=i+1)
assert selected == dict(threshold=.20381558,right=43,accepted=86)
report = dict(verified_provenance_files=82, archived_original_files=20,
              archived_original_bytes=277065, complete_rows=10570,
              official_scores=scores, corrected_transitions=transitions,
              worker_selected_transition_corrections=corrections,
              calibration=selected, legacy_predictions_byte_identical=True,
              note='Original worker transition artifacts preserved; official aggregate metrics unchanged. Three refusals were mislabeled right-to-right by worker helper.',
              decision='REJECTED:2631 rawcorrect below2638;694 selectedcorrect below815. No retraining or inference repeated.')
(HERE/'reports/parent-review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
