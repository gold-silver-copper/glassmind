"""Independent TRAIN joins, native-score checks and complete CAL overlay replay."""
from pathlib import Path
import collections
import hashlib
import itertools
import json
import math
import tarfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SUB = ROOT/'contextledger-run/terminal-originals/worktree/tools/dialogue/context-ledger'
ART = SUB/'artifacts'
SOURCE = ROOT/'entityledger-source'
read = lambda p: json.loads(p.read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

manifest = read(SUB/'archive-manifest.json')
with tarfile.open(SUB/'archive.tar.gz') as archive:
    for name, expected in manifest.items():
        member = archive.getmember(name)
        assert member.isfile() and not Path(name).is_absolute() and '..' not in Path(name).parts
        stream = archive.extractfile(member)
        assert stream is not None
        raw = stream.read()
        assert len(raw) == expected['bytes'] and hashlib.sha256(raw).hexdigest() == expected['sha256'], name
source_manifest = read(SOURCE/'manifest.json')
for name, expected in source_manifest.items():
    assert (SOURCE/name).stat().st_size == expected['bytes'] and sha(SOURCE/name) == expected['sha256'], name
CONTEXT = ROOT/'contextledger-source'
for name, identity in read(CONTEXT/'manifest.json').items():
    assert (CONTEXT/name).stat().st_size == identity['bytes'] and sha(CONTEXT/name) == identity['sha256']

raw_data = sorted(read(SOURCE/'train.json'),key=lambda d:d['dialogue_id'])
baseline = read(CONTEXT/'baseline-train.json')
inputs = read(CONTEXT/'train-input.json')
assert len(raw_data) == len(baseline) == len(inputs) == 3174
data = {d['dialogue_id']: d for d in raw_data}
base = {}
for d, prediction, runtime in zip(raw_data, baseline, inputs, strict=True):
    assert runtime == [{k:t[k] for k in ['speaker','utterance']} for t in d['turns']]
    indexed = {p['turn_index']:p for p in prediction}
    assert len(indexed) == len(prediction)
    assert set(indexed) == {i for i,t in enumerate(d['turns']) if t['speaker']=='USER'}
    base[d['dialogue_id']] = indexed
is_cal = lambda name: int(hashlib.sha256(name.encode()).hexdigest(),16) % 5 == 0
assert sum(is_cal(name) for name in data) == 642
schemas = {s['service_name']:s for s in read(SOURCE/'schema.json')}
full_schema = read(SOURCE/'full-train-schema.json')
allowed_roles = {s['service_name']+'/'+slot['name'] for s in full_schema
                 for slot in s['slots'] if not slot['is_categorical']}
# Reconstruct actual-span role/NONE targets directly from original TRAIN.
import numpy as np
model = read(ART/'model.json')
assert model['boundary'] == read(CONTEXT/'rejected-model.json')['boundary']
for component in ['boundary','role','rank']:
    assert all(math.isfinite(x) for x in model[component]['w'])
role_weights = np.array(model['role']['w'],dtype=np.float64)
role_indices = np.arange(len(model['roles']),dtype=np.uint64)*np.uint64(2654435761)
source_counts = collections.Counter()
frequencies = collections.Counter()
max_role_error = 0.0
role_rows = []
def fnv(text):
    value = 2166136261
    for byte in text.encode(): value = ((value ^ byte)*16777619)&0xffffffff
    return value
for line in (ART/'source-spans.jsonl').open():
    entry = json.loads(line); row = entry['row']; d = data[row['id']]
    ti = row['turn']; turn = d['turns'][ti]; start,end = row['raw']['start'],row['raw']['end']
    assert turn['speaker']=='SYSTEM' and row['source']==turn['utterance']
    assert row['cal']==is_cal(row['id']) and 0<=start<end<=len(row['source'].encode())
    row['source'].encode()[start:end].decode()
    previous = next((t['utterance'] for t in reversed(d['turns'][:ti]) if t['speaker']=='USER'),'')
    assert row['previous_user']==previous
    roles = set()
    for frame in turn['frames']:
        for span in frame.get('slots',[]):
            a,b = span['start'],span['exclusive_end']
            if (len(turn['utterance'][:a].encode()),len(turn['utterance'][:b].encode()))==(start,end):
                roles.add(frame['service']+'/'+span['slot'])
    assert sorted(roles)==row['exact_roles']
    if not row['cal'] and len(roles)==1: frequencies[next(iter(roles))]+=1
    expected = next(iter(roles)) if len(roles)==1 and next(iter(roles)) in model['roles'] else 'NONE'
    assert row['label']==expected
    feature_ids = sorted(set(map(fnv,row['context'])))
    assert feature_ids==row['features']
    indices = (np.array(feature_ids,dtype=np.uint64)[None,:] ^ role_indices[:,None]) & 262143
    scores = role_weights[indices].sum(axis=1)
    delta = float(np.max(np.abs(scores-np.array(entry['scores']))))
    assert delta<0.002; max_role_error=max(max_role_error,delta)
    best = entry['prediction']; assert best==max(range(len(entry['scores'])),key=lambda i:entry['scores'][i])
    margin = entry['scores'][best]-max(x for i,x in enumerate(entry['scores']) if i!=best)
    assert abs(margin-entry['margin'])<0.002
    assert entry['enters_ledger']==(model['roles'][best]!='NONE')
    source_counts['source_rows']+=1;source_counts['class_scores']+=len(scores)
    source_counts['CAL_rows' if row['cal'] else 'FIT_rows']+=1
    source_counts['CAL_NONE_targets' if row['cal'] else 'FIT_NONE_targets']+=expected=='NONE'
    role_rows.append((row['id'],ti,start,end,model['roles'][best],entry['margin']))
expected_roles = ['NONE']+[name for name,count in sorted(frequencies.items(),key=lambda z:(-z[1],z[0])) if count>=20][:127]
assert model['roles']==expected_roles and len(expected_roles)==128
source_predictions = {(d,ti,a,b):(role,margin) for d,ti,a,b,role,margin in role_rows}
weights = model['rank']['w']
assert len(weights) == 262144 and (ART/'model.json').stat().st_size <= 16*1024*1024
proposals = {}
verified = collections.Counter()
max_error = 0.0
for line in (ART/'proposals.jsonl').open():
    row = json.loads(line)
    name, ti, service, slot = row['id'],row['turn'],row['service'],row['slot']
    d = data[name]
    assert row['cal'] == is_cal(name)
    state = base[name][ti]
    assert service in state['active_services']
    old = next(f for f in state['frames'] if f['service']==service)
    assert slot not in old['state']['slot_values']
    gold = next((f['state']['slot_values'].get(slot,[]) for f in d['turns'][ti]['frames']
                 if f['service']==service),[])
    assert row['gold'] == gold
    for candidate in row['candidates']:
        entity = candidate['entity']
        si = entity['source_turn']
        source = d['turns'][si]
        assert si < ti and source['speaker']=='SYSTEM' and source['utterance']==entity['source']
        assert source['utterance'].encode()[entity['start']:entity['end']].decode()==entity['value']
        role, margin = source_predictions[name,si,entity['start'],entity['end']]
        assert entity['role']==role!='NONE' and abs(entity['confidence']-margin)<0.002
        assert sum(t['speaker']=='SYSTEM' for t in d['turns'][si+1:ti]) < 32
        assert candidate['correct'] == any(v.lower()==entity['value'].lower() for v in gold)
        score = sum(weights[f & 262143] for f in candidate['features'])
        error = abs(score-candidate['score'])
        assert error < 0.001
        max_error = max(max_error,error)
        verified['candidate_scores_and_sources'] += 1
    verified['CAL_opportunities' if row['cal'] else 'FIT_opportunities'] += 1
    if row['cal']:
        key = (name,ti,service,slot)
        assert key not in proposals
        proposals[key] = row['candidates']

decisions = {}
for line in (ART/'decisions.jsonl').open():
    row = json.loads(line)
    grid = tuple(row['grid'])
    key = (row['id'],row['turn'],row['service'],row['slot'])
    candidates = proposals[key]
    eligible = [i for i,c in enumerate(candidates) if c['entity']['confidence'] >= grid[0]]
    chosen = None
    if eligible:
        best = max(eligible,key=lambda i:candidates[i]['score'])
        value = candidates[best]['entity']['value'].lower()
        alternative = max([0.0]+[c['score'] for c in candidates if c['entity']['value'].lower()!=value])
        score = candidates[best]['score']
        if score >= grid[1] and score-alternative >= grid[2]:
            chosen = best
    assert row['chosen'] == chosen
    assert (grid,key) not in decisions
    decisions[grid,key] = chosen
    verified['decisions'] += 1
grids = list(itertools.product([0.,2.],[2.,4.,8.,16.],[2.,4.]))
assert len(decisions) == len(proposals)*len(grids) == 22912

def exact(gold, predicted, schema):
    for slot in schema['slots']:
        name = slot['name']
        if name not in gold:
            if name in predicted: return False
        elif name not in predicted:
            return False
        elif slot['is_categorical']:
            if gold[name][0].lower()!=predicted[name][0].lower(): return False
        elif predicted[name][0] not in gold[name]:
            return False
    return True

reports = []
worker = {tuple(g['grid']):g for g in read(ART/'cal-report.json')['grid']}
for grid in grids:
    count = collections.Counter()
    lost_joint, lost_frames = [],[]
    for d in raw_data:
        name = d['dialogue_id']
        if not is_cal(name): continue
        held = {}
        for ti,turn in enumerate(d['turns']):
            if turn['speaker']!='USER': continue
            before = base[name][ti]
            frames = {f['service']:f['state'] for f in before['frames']}
            gold = {f['service']:f['state'] for f in turn['frames'] if f['service'] in schemas}
            held = {k:v for k,v in held.items() if k[1] not in frames[k[0]]['slot_values']}
            for service in before['active_services']:
                for slot in schemas[service]['slots']:
                    key = (name,ti,service,slot['name'])
                    if key not in proposals or (service,slot['name']) in held: continue
                    chosen = decisions[grid,key]
                    if chosen is not None:
                        held[service,slot['name']] = proposals[key][chosen]['entity']['value']
                        count['initial_events'] += 1
            changed = {service:dict(state['slot_values']) for service,state in frames.items()}
            for (service,slot),value in held.items():
                changed[service][slot] = [value]
                kind = ('held_unscored' if service not in gold else 'correct_scored'
                        if value in gold[service]['slot_values'].get(slot,[]) else 'wrong_scored')
                count[kind] += 1
                count['correct_inactive'] += kind=='correct_scored' and service not in before['active_services']
                count['changed_slot_turns'] += 1
                count['inactive_changes'] += service not in before['active_services']
            prior_ok = after_ok = set(before['active_services'])==set(gold)
            count['user_turns'] += 1
            for service,state in gold.items():
                count['gold_frames'] += 1
                was = exact(state['slot_values'],frames[service]['slot_values'],schemas[service])
                now = exact(state['slot_values'],changed[service],schemas[service])
                count['baseline_correct_frames'] += was
                count['candidate_correct_frames'] += now
                if was and not now: lost_frames.append([name,ti,service])
                intent = state['active_intent']==frames[service]['active_intent']
                prior_ok = prior_ok and was and intent
                after_ok = after_ok and now and intent
            count['baseline_correct_joint'] += prior_ok
            count['candidate_correct_joint'] += after_ok
            if prior_ok and not after_ok: lost_joint.append([name,ti])
    expected = worker[grid]
    mapping = {'new_correct_fills':count['correct_scored']-count['correct_inactive'],
               'new_wrong_assertions':count['wrong_scored'], 'held_unscored':count['held_unscored'],
               'changed_slot_turns':count['changed_slot_turns'], 'scored_correct_inactive_no_credit':count['correct_inactive'],
               'user_turns':count['user_turns'], 'gold_frames':count['gold_frames'],
               'joint_correct':count['candidate_correct_joint'], 'strict_frames':count['candidate_correct_frames'],
               'lost_joint':len(lost_joint), 'lost_frames':len(lost_frames),
               'predicted_inactive_changes':count['inactive_changes']}
    for key,value in mapping.items(): assert value==expected[key],(grid,key,value,expected[key])
    tag='-'.join(str(int(v)) for v in grid)
    assert count['initial_events']==len(read(ART/f'cal-{tag}-events.json'))
    assert count['baseline_correct_frames']==1478 and count['baseline_correct_joint']==5803
    passed = mapping['new_correct_fills']>=30 and count['wrong_scored']==0 and not lost_joint and not lost_frames
    assert passed==expected['passes']==False
    reports.append(dict(grid=grid,counts=dict(count),lost_joint=lost_joint,lost_frames=lost_frames,passed=passed))
result = dict(archive_files_verified=len(manifest),source_files_verified=len(source_manifest),
              all_train_runtime_inputs_verified=3174,baseline_inputs_exactly_aligned=True,
              source_checks=dict(source_counts),max_role_error=max_role_error,
              FIT_role_labels=len(expected_roles),verified=dict(verified),max_rank_error=max_error,
              grids=reports,result='REJECTED_CAL; all 16 original gates independently reproduced',
              limitation='Frozen-state overlay; not integrated candidate inference or DEV accuracy')
(HERE/'cal-review.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k!='grids'},indent=2))
