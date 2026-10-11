from pathlib import Path
import collections
import hashlib
import json

HERE = Path(__file__).resolve().parent
data = json.loads((HERE / 'train.json').read_text())
proposals = json.loads((HERE / 'history-proposals.json').read_text())
errors = json.loads((HERE / 'train-errors.json').read_text())
initial = {(e['dialogue_id'], e['turn_index'], e['service'], e['slot'])
           for e in errors if e['operation'] == 'set_gold' and e['kind'] == 'missing'}
counts = collections.Counter()
initial_counts = collections.Counter()
seen_initial = set()
rows = []
for row in proposals:
    dialogue = data[row['conversation_index']]
    turns = dialogue['turns']
    turn = turns[row['turn_index']]
    assert turn['speaker'] == 'USER'
    gold = next((f for f in turn['frames'] if f['service'] == row['service']), None)
    expected = gold['state']['slot_values'].get(row['slot'], []) if gold else []
    for proposal in row['proposals']:
        assert proposal['source_turn'] < row['turn_index']
        source = turns[proposal['source_turn']]
        assert source['speaker'] == 'SYSTEM'
        source_bytes = source['utterance'].encode()
        assert source_bytes[proposal['start']:proposal['end']].decode() == proposal['value']
        assert (proposal['service'], proposal['slot']) == (row['service'], row['slot'])
    # Distinct verbatim values, retaining case distinctions from the strict metric.
    values = sorted({p['value'] for p in row['proposals']})
    includes_gold = bool(set(values) & set(expected))
    category = ('unscored_service' if gold is None else
                'unique_correct' if len(values) == 1 and includes_gold else
                'unique_wrong' if len(values) == 1 else
                'multiple_with_gold' if includes_gold else 'multiple_without_gold')
    counts[category] += 1
    key = (dialogue['dialogue_id'], row['turn_index'], row['service'], row['slot'])
    if key in initial:
        seen_initial.add(key)
        initial_counts[category] += 1
    rows.append(dict(dialogue_id=key[0], turn_index=key[1], service=key[2], slot=key[3],
                     values=values, gold=expected, category=category))
initial_counts['no_proposal'] = len(initial - seen_initial)
report = dict(proposal_rows=len(proposals), classifications=dict(counts),
              initial_missing_set_events=len(initial), initial_classifications=dict(initial_counts),
              provenance='Every native UTF-8 source span verified against utterance-only input',
              limitation='Candidate coverage only; no history candidate was applied to session state')
(HERE / 'history-audit.json').write_text(json.dumps(report, indent=2) + '\n')
(HERE / 'history-classifications.json').write_text(json.dumps(rows, ensure_ascii=False, indent=2) + '\n')
(HERE / 'history-artifacts.json').write_text(json.dumps({str(p):dict(bytes=p.stat().st_size,
    sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in [
        HERE/'history-proposals.json', HERE/'target/release/glassmind-history-probe',
        HERE/'history-audit.py', HERE/'history-audit.json', HERE/'history-classifications.json']}, indent=2) + '\n')
print(json.dumps(report, indent=2))
