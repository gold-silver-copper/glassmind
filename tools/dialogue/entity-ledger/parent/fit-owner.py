from pathlib import Path
import collections
import hashlib
import json

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ART = ROOT/'entityledger-run/terminal-originals/worktree/tools/dialogue/entity-ledger/artifacts'
source = json.loads((ROOT/'entityledger-source/train.json').read_text())
baseline = json.loads((ART/'baseline-train.json').read_text())
data = {d['dialogue_id']:d for d in source}
pred = {d['dialogue_id']:{r['turn_index']:r for r in rows}
        for d,rows in zip(source,baseline,strict=True)}
seen = {}
counts = collections.Counter()
supported = collections.Counter()
for line in (ART/'proposals.jsonl').open():
    row = json.loads(line)
    if row['cal']: continue
    assert int(hashlib.sha256(row['id'].encode()).hexdigest(),16)%5 != 0
    turns = data[row['id']]['turns']
    for candidate in row['candidates']:
        entity = candidate['entity']
        key = (row['id'],entity['source_turn'],entity['start'],entity['end'])
        identity = (entity['role'],entity['value'])
        if key in seen:
            assert seen[key] == identity
            continue
        seen[key] = identity
        si = entity['source_turn']
        utterance = turns[si]['utterance']
        roles = set()
        for frame in turns[si]['frames']:
            for span in frame.get('slots',[]):
                start = len(utterance[:span['start']].encode())
                end = len(utterance[:span['exclusive_end']].encode())
                if (start,end)==(entity['start'],entity['end']):
                    roles.add(frame['service']+'/'+span['slot'])
        before = [i for i in pred[row['id']] if i<si]
        active = set(pred[row['id']][max(before)]['active_services']) if before else set()
        owner_ok = entity['role'].split('/',1)[0] in active
        label = 'exact_role' if entity['role'] in roles else 'wrong_role' if roles else 'unaligned_span'
        key2 = label + ('_owner_compatible' if owner_ok else '_owner_incompatible')
        counts[key2] += 1
        if any(role.split('/',1)[0] in active for role in roles):
            supported[key2] += 1
result = dict(split='FIT only; distinct extracted source entities used by missing-slot opportunities',
              entities=len(seen),all_entities=dict(counts),
              annotated_owner_in_predicted_context=dict(supported),
              limitation='Source-role diagnostic only. No candidate state, CAL gate, model, or threshold changed.')
(HERE/'fit-owner.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
