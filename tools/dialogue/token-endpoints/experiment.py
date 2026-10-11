"""One frozen TRAIN-only endpoint experiment. Never a runtime/scoring dependency."""
import argparse
import collections
import hashlib
import json
import pathlib
import re

TOKEN = re.compile(r"[^\W_]+(?:['’.-][^\W_]+)*", re.UNICODE)


def split(d):
    return 'CAL' if int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(), 16) % 5 == 0 else 'FIT'


def shape(word):
    out = []
    for c in word:
        cls = 'U' if c.isupper() else 'L' if c.islower() else 'D' if c.isnumeric() else 'A' if c.isalpha() else c
        if out and out[-1][0] == cls and cls in ('U', 'L', 'D', 'A'):
            out[-1][1] += 1
        else:
            out.append([cls, 1])
    result = ''.join(c + str(min(n, 3)) if c in ('U', 'L', 'D', 'A') else c for c, n in out)
    return result if len(result.encode()) <= 64 else None


def gap(text):
    value = re.sub(r'\s+', 'W', text)
    return value if len(value) <= 3 else None


def latest(value, history):
    if not value or len(value.split()) > 12:
        return None
    for ti in range(len(history) - 1, -1, -1):
        source = history[ti]
        occurrences = [m.start() for m in re.finditer(re.escape(value), source)
                       if (m.start() == 0 or not source[m.start() - 1].isalnum())
                       and (m.end() == len(source) or not source[m.end()].isalnum())]
        if occurrences:
            start = occurrences[-1]
            return ti, start, start + len(value)
    return None


def context(source, start, end):
    if len(source.encode()) > 4096:
        return None
    tokens = list(TOKEN.finditer(source))
    if len(tokens) > 256:
        return None
    inside = [i for i, t in enumerate(tokens) if t.start() >= start and t.end() <= end]
    if not inside or len(inside) > 12 or tokens[inside[0]].start() != start or tokens[inside[-1]].end() != end:
        return None
    tail = inside[-1]
    following = tokens[tail + 1:tail + 3]
    if not following:
        return None
    source_shapes = [shape(tokens[i].group()) for i in inside[-2:]]
    source_shapes = ['EMPTY'] * (2 - len(source_shapes)) + source_shapes
    next_shapes = [shape(t.group()) for t in following] + ['END'] * (2 - len(following))
    gaps = [gap(source[end:following[0].start()]),
            gap(source[following[0].end():following[1].start()]) if len(following) == 2 else 'END']
    parts = source_shapes + next_shapes + gaps
    if None in parts:
        return None
    return tuple(parts), [t.end() for t in following]


def discover(dialogues, schemas):
    slots = {s['service_name']: {q['name'] for q in s['slots'] if not q['is_categorical']} &
             {n for i in s['intents'] for n in i['required_slots'] + list(i['optional_slots'])} for s in schemas}
    candidates = collections.defaultdict(lambda: collections.defaultdict(set))
    counts = collections.Counter()
    for d in dialogues:
        if split(d) != 'FIT':
            continue
        history = []
        for ti, t in enumerate(d['turns']):
            source = t['utterance']
            history.append(source)
            tokens = list(TOKEN.finditer(source))
            for f in t['frames']:
                for s in f.get('slots', []):
                    if s['slot'] not in slots.get(f['service'], set()):
                        continue
                    start, end = s['start'], s['exclusive_end']
                    inside = [tok for tok in tokens if tok.start() >= start and tok.end() <= end]
                    if not inside or inside[0].start() != start or inside[-1].end() != end:
                        counts['unaligned_gold_spans'] += 1
                        continue
                    for extension in (1, 2):
                        if len(inside) <= extension:
                            continue
                        prefix_end = inside[-extension - 1].end()
                        if latest(source[start:prefix_end], history) != (ti, start, prefix_end):
                            counts['nonlatest_prefix'] += 1
                            continue
                        ctx = context(source, start, prefix_end)
                        if ctx is not None and len(ctx[1]) >= extension and ctx[1][extension - 1] == end:
                            candidates[ctx[0]][extension].add(d['dialogue_id'])
                            counts['prefix_discovery_rows'] += 1
    rows = [{'key': list(key), 'lengths': {str(n): sorted(ids) for n, ids in sorted(lengths.items())}}
            for key, lengths in sorted(candidates.items())]
    return rows, dict(counts)


def endpoints(turn, start, sources):
    return sorted({s['exclusive_end'] for f in turn['frames'] for s in f.get('slots', [])
                   if [f['service'], s['slot']] in sources and s['start'] == start})


def run(root, trace, output):
    dialogues = json.loads((root / 'selected/train.json').read_text())
    schemas = json.loads((root / 'train/schema.json').read_text())
    candidates = json.loads((output / 'candidates.json').read_text())
    by_key = {tuple(c['key']): c for c in candidates}
    stats = collections.defaultdict(lambda: {'FIT': {'ids': set(), 'rows': 0, 'mismatches': 0},
                                           'CAL': {'ids': set(), 'rows': 0, 'mismatches': 0}})
    totals = collections.Counter()
    audit = []
    proposal_instances = collections.defaultdict(set)
    for p in map(json.loads, trace.read_text().splitlines()):
        d = dialogues[p['conversation']]
        partition = split(d)
        row = dict(p, dialogue_id=d['dialogue_id'], partition=partition, proposals=[])
        totals[partition + '_reference_proposals'] += 1
        history = [t['utterance'] for t in d['turns'][:p['turn']]]
        loc = latest(p['value'], history)
        if p['punctuation_value'] is not None:
            row['status'] = 'existing_punctuation_priority'
        elif loc is None:
            row['status'] = 'no_whole_exact_occurrence'
        else:
            ti, start, end = loc
            source = history[ti]
            row['source_turn'] = ti
            row['start'] = len(source[:start].encode())
            row['original_end'] = len(source[:end].encode())
            ctx = context(source, start, end)
            if ctx is None:
                row['status'] = 'no_bounded_token_context'
            elif ctx[0] not in by_key:
                row['status'] = 'unknown_key'
                row['key'] = list(ctx[0])
            else:
                key, ends = ctx
                row['key'] = list(key)
                candidate = by_key[key]
                row['status'] = 'conflicting_lengths' if len(candidate['lengths']) > 1 else 'candidate_key'
                gold_ends = endpoints(d['turns'][ti], start, p['sources'])
                for length in map(int, candidate['lengths']):
                    if len(ends) < length:
                        continue
                    new_end = ends[length - 1]
                    correct = gold_ends == [new_end]
                    info = {'length': length, 'end': len(source[:new_end].encode()),
                            'value': source[start:new_end], 'endpoint_match': correct,
                            'gold_ends': [len(source[:e].encode()) for e in gold_ends],
                            'missing_annotation': not gold_ends}
                    row['proposals'].append(info)
                    stat = stats[(key, length)][partition]
                    stat['ids'].add(d['dialogue_id']); stat['rows'] += 1
                    stat['mismatches'] += not correct
                    proposal_instances[(key, length, partition)].add((p['conversation'], p['turn'], p['service'], p['slot']))
                    totals[partition + '_generated_proposals'] += 1
                    totals[partition + '_endpoint_mismatches'] += not correct
                    totals[partition + '_missing_annotations'] += not gold_ends
        totals[row['status']] += 1
        audit.append(row)
    rules = []
    cal_instances = set()
    for candidate in candidates:
        key = tuple(candidate['key'])
        for length in map(int, candidate['lengths']):
            stat = stats[(key, length)]
            reasons = []
            if len(candidate['lengths']) > 1:
                reasons.append('conflicting_FIT_discovery_lengths')
            for partition, minimum in [('FIT', 5), ('CAL', 3)]:
                if len(stat[partition]['ids']) < minimum:
                    reasons.append(partition + '_insufficient_actual_conversations')
                if stat[partition]['mismatches']:
                    reasons.append(partition + '_endpoint_mismatches')
            rules.append({'key': list(key), 'length': length,
                          **{s: {'conversations': len(v['ids']), 'rows': v['rows'], 'mismatches': v['mismatches']}
                             for s, v in stat.items()}, 'accepted': not reasons, 'rejections': reasons})
            if not reasons:
                cal_instances |= proposal_instances[(key, length, 'CAL')]
    accepted = [r for r in rules if r['accepted']]
    report = {'decision': 'TRAIN_GATE_PASS' if accepted and len(cal_instances) >= 30 else 'REJECT_BEFORE_DEV',
              'candidate_keys': len(candidates), 'candidate_key_lengths': len(rules),
              'qualifying_rules': len(accepted), 'qualifying_CAL_instances': len(cal_instances),
              'totals': dict(totals), 'rules': rules,
              'limitation': 'Frozen classifiers used all TRAIN including CAL; empirical new-rule validation only.'}
    (output / 'audit.jsonl').write_text(''.join(json.dumps(row, ensure_ascii=False) + '\n' for row in audit))
    (output / 'gate.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'rules'}, indent=2))


def main():
    p = argparse.ArgumentParser()
    p.add_argument('mode', choices=['discover', 'audit'])
    p.add_argument('--root', type=pathlib.Path, required=True)
    p.add_argument('--output', type=pathlib.Path, required=True)
    p.add_argument('--trace', type=pathlib.Path)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    if a.mode == 'discover':
        rows, counts = discover(json.loads((a.root / 'selected/train.json').read_text()),
                                json.loads((a.root / 'train/schema.json').read_text()))
        (a.output / 'candidates.json').write_text(json.dumps(rows, indent=2) + '\n')
        (a.output / 'discovery.json').write_text(json.dumps(counts, indent=2) + '\n')
        print(len(rows), counts)
    else:
        run(a.root, a.trace, a.output)


if __name__ == '__main__':
    main()
