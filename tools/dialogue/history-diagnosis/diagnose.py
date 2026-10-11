from pathlib import Path
import collections
import hashlib
import json
import sys
import warnings

HERE = Path(__file__).resolve().parent
REPO = Path('/home/gsc/code/glassmind')
sys.path.insert(0, str(REPO / 'crates/dialogue/tools'))
with warnings.catch_warnings():
    warnings.simplefilter('ignore')
    import upstream_metrics as metrics

for name, identity in json.loads((HERE / 'freeze.json').read_text()).items():
    assert hashlib.sha256(Path(name).read_bytes()).hexdigest() == identity['sha256'], name
data = json.loads((HERE / 'train.json').read_text())
predictions = json.loads((HERE / 'full/predictions.json').read_text())
bundle = HERE.parent / 'draftsession-independent/bundles/valid'
schemas = {s['service_name']: s for s in json.loads((bundle / 'schema.json').read_text())}
totals = collections.Counter()
overlap = collections.Counter()
slot_errors = collections.Counter()
operations = collections.Counter()
errors = []
for dialogue, predictions_for_dialogue in zip(data, predictions, strict=True):
    by_turn = {p['turn_index']: p for p in predictions_for_dialogue}
    prior_gold = {}
    prior_predicted = {}
    for index, turn in enumerate(dialogue['turns']):
        if turn['speaker'] != 'USER':
            continue
        snapshot = by_turn.pop(index)
        predicted = {f['service']: f for f in snapshot['frames']}
        gold = {f['service']: f for f in turn['frames']}
        flags = set()
        if set(snapshot['active_services']) != set(gold):
            flags.add('route')
        for service, frame in gold.items():
            state = frame['state']
            proposed = predicted[service]['state']
            if state['active_intent'] != proposed['active_intent']:
                flags.add('intent')
            gv, pv = state['slot_values'], proposed['slot_values']
            correctness, _, _ = metrics.compare_slot_values(gv, pv, schemas[service], False)
            for slot, correct in zip(schemas[service]['slots'], correctness, strict=True):
                if correct:
                    continue
                flags.add('slots')
                name = slot['name']
                kind = 'missing' if name not in pv else 'spurious' if name not in gv else 'wrong'
                role = 'categorical' if slot['is_categorical'] else 'text'
                slot_errors[(service, name, role, kind)] += 1
                previous_gold = prior_gold.get(service, {}).get(name)
                previous_predicted = prior_predicted.get(service, {}).get(name)
                operation = ('unchanged_gold' if previous_gold == gv.get(name)
                             else 'delete_gold' if name not in gv
                             else 'set_gold')
                persisted_prediction = previous_predicted == pv.get(name)
                operations[(service, name, operation, persisted_prediction)] += 1
                errors.append(dict(dialogue_id=dialogue['dialogue_id'], turn_index=index,
                                   service=service, slot=name, role=role, kind=kind,
                                   gold=gv.get(name), predicted=pv.get(name),
                                   previous_gold=previous_gold, previous_predicted=previous_predicted,
                                   operation=operation, persisted_prediction=persisted_prediction,
                                   utterance=turn['utterance']))
            prior_gold[service] = gv
        prior_predicted = {s: f['state']['slot_values'] for s, f in predicted.items()}
        totals['user_turns'] += 1
        totals['gold_frames'] += len(gold)
        overlap['+'.join(sorted(flags)) or 'correct'] += 1
    assert not by_turn
report = dict(totals=dict(totals), turn_error_overlap=dict(overlap),
              slot_errors=[dict(service=k[0], slot=k[1], role=k[2], kind=k[3], count=v)
                           for k, v in slot_errors.most_common()],
              gold_operations=[dict(service=k[0], slot=k[1], operation=k[2],
                                    persisted_prediction=k[3], count=v)
                               for k, v in operations.most_common()])
official = json.loads((HERE / 'full/report.json').read_text())['all']
assert totals['user_turns'] == official['user_turns']
assert totals['gold_frames'] == official['gold_frames']
assert overlap['correct'] == official['full_joint_correct']
(HERE / 'diagnosis.json').write_text(json.dumps(report, indent=2) + '\n')
(HERE / 'train-errors.json').write_text(json.dumps(errors, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(report, indent=2))
