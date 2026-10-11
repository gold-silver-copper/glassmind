#!/usr/bin/env python3
"""Frozen raw FORM evaluator. No normalization, subset gate, or gold at inference."""
import argparse
import collections
import hashlib
import json
import pathlib
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def write_json(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')


def valid_tree(p):
    n = len(p.get('forms', []))
    hs, ls = p.get('heads', []), p.get('deprels', [])
    if len(hs) != n or len(ls) != n or len(p.get('upos', [])) != n or hs.count(0) != 1:
        return False
    for i, (h, label) in enumerate(zip(hs, ls), 1):
        if not isinstance(h, int) or not 0 <= h <= n or h == i or (h == 0) != (label == 'root'):
            return False
        seen, j = set(), i
        while j:
            if j in seen or not 1 <= j <= n:
                return False
            seen.add(j)
            j = hs[j - 1]
    return True


def source_valid(text, p):
    if p.get('status') != 'predicted':
        return False
    forms, spans, raw = p['forms'], p['spans'], text.encode()
    if not 1 <= len(forms) <= 128 or len(raw) > 16384 or len(spans) != len(forms):
        return False
    end = 0
    try:
        for form, span in zip(forms, spans):
            a, b = span['start'], span['end']
            if not end <= a < b <= len(raw):
                return False
            if not all(c.isspace() for c in raw[end:a].decode()):
                return False
            if raw[a:b].decode() != form or form != span['form'] or any(c.isspace() for c in form):
                return False
            end = b
        return all(c.isspace() for c in raw[end:].decode()) and valid_tree(p)
    except (UnicodeDecodeError, KeyError, TypeError):
        return False


def train_keys(path):
    keys = set()
    for block in path.read_text().strip().split('\n\n'):
        rows = [line.split('\t') for line in block.splitlines() if not line.startswith('#')]
        keys.add(tuple(row[1] for row in rows if row[0].isdigit()))
    return keys


def predict(args, inputs, output):
    subprocess.run([str(args.binary), 'predict', '--model', str(args.baseline/'model.bin'),
                    '--input', str(inputs), '--output', str(output)], check=True, timeout=600)
    return read_rows(output)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline', type=pathlib.Path, required=True)
    parser.add_argument('--source', type=pathlib.Path, required=True)
    parser.add_argument('--binary', type=pathlib.Path, required=True)
    parser.add_argument('--out', type=pathlib.Path, required=True)
    parser.add_argument('--mode', choices=['sample', 'full', 'authored', 'forms'], required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    output = args.out/(args.mode+'-predictions.jsonl')
    if args.mode == 'forms':
        predict(args, args.baseline/'forms.jsonl', output)
        reference = args.baseline/'forms-predictions.jsonl'
        result = {'rows': len(read_rows(output)), 'byte_identical': output.read_bytes() == reference.read_bytes(),
                  'baseline_sha256': sha(reference), 'candidate_sha256': sha(output)}
        write_json(args.out/'forms-score.json', result)
        assert result['rows'] == 2001 and result['byte_identical'], result
        print(json.dumps(result))
        return
    if args.mode == 'authored':
        gold = read_rows(pathlib.Path(__file__).with_name('authored.jsonl'))
        inputs = args.out/'authored-inputs.jsonl'
        inputs.write_text(''.join(json.dumps({'text': g['text']}, ensure_ascii=False)+'\n' for g in gold))
        preds = predict(args, inputs, output)
        assert len(preds) == len(gold)
        rows = [{'index': i, 'exact': p.get('forms') == g['forms'], 'source_valid': source_valid(g['text'], p),
                 'text': g['text'], 'expected': g['forms'], 'actual': p.get('forms')} for i, (g, p) in enumerate(zip(gold, preds), 1)]
        result = {'cases': len(rows), 'exact': sum(r['exact'] for r in rows),
                  'source_valid': sum(r['source_valid'] for r in rows), 'rows': rows}
        write_json(args.out/'authored-score.json', result)
        print(json.dumps({k: v for k, v in result.items() if k != 'rows'}))
        return
    gold = json.loads((args.baseline/'raw/gold.json').read_text())
    before = read_rows(args.baseline/'raw/predictions.jsonl')
    inputs_all = read_rows(args.baseline/'raw/inputs.jsonl')
    assert len(gold) == len(before) == len(inputs_all) == 2001
    count = 100 if args.mode == 'sample' else 2001
    inputs = args.out/(args.mode+'-inputs.jsonl')
    # Only raw text crosses into inference; annotations stay in this evaluator.
    assert all(set(x) == {'text'} for x in inputs_all)
    assert all(x['text'] == g['text'] for x, g in zip(inputs_all, gold))
    inputs.write_text(''.join(json.dumps(x, ensure_ascii=False)+'\n' for x in inputs_all[:count]))
    after = predict(args, inputs, output)
    assert len(after) == count
    train = train_keys(args.source/'en_ewt-ud-train.conllu')
    frequencies = collections.Counter((g['text'], tuple(g['forms'])) for g in gold)
    groups = collections.defaultdict(collections.Counter)
    transitions = collections.Counter()
    ledger, remaining = [], []
    tokens = collections.Counter()
    for i, (g, b, a) in enumerate(zip(gold[:count], before[:count], after), 1):
        old, new = b.get('forms') == g['forms'], a.get('forms') == g['forms']
        transition = ('exact' if old else 'mismatch')+'->'+('exact' if new else 'mismatch')
        transitions[transition] += 1
        source_ok = source_valid(g['text'], a)
        row = {'index': i, 'transition': transition, 'changed_forms': b.get('forms') != a.get('forms'),
               'source_valid': source_ok, 'baseline_status': b.get('status'), 'candidate_status': a.get('status'),
               'text': g['text'], 'gold_forms': g['forms'], 'baseline_forms': b.get('forms'),
               'candidate_forms': a.get('forms')}
        ledger.append(row)
        if not new:
            remaining.append(row)
        group_names = ['all', 'train_duplicate' if tuple(g['forms']) in train else 'train_nonduplicate',
                       'dev_repeated' if frequencies[(g['text'], tuple(g['forms']))] > 1 else 'dev_singleton']
        for group in group_names:
            groups[group].update(sentences=1, baseline_exact=int(old), candidate_exact=int(new),
                                 preserved=int(old and new), lost=int(old and not new), gained=int(new and not old),
                                 valid_source_and_tree=int(source_ok), predicted=int(a.get('status') == 'predicted'))
        tokens.update(baseline=len(b.get('forms', [])), candidate=len(a.get('forms', [])), gold=len(g['forms']))
    gate = groups['all']['candidate_exact'] >= 1470 and groups['all']['lost'] == 0 and groups['all']['valid_source_and_tree'] == 2001 and groups['all']['predicted'] == 2001
    result = {'mode': args.mode, 'groups': dict(groups), 'transitions': dict(transitions), 'tokens': dict(tokens),
              'changed_form_sequences': sum(r['changed_forms'] for r in ledger),
              'new_refusals': sum(r['baseline_status'] == 'predicted' and r['candidate_status'] != 'predicted' for r in ledger),
              'raw_gate': gate if args.mode == 'full' else None,
              'prediction_sha256': sha(output), 'binary_sha256': sha(args.binary),
              'metric': 'Exact integer-word FORM sequence; no normalization. Duplicate subgroups retain official total. Not dependency or semantic accuracy.'}
    write_json(args.out/(args.mode+'-score.json'), result)
    for name, rows in [('transitions', ledger), ('remaining-mismatches', remaining)]:
        (args.out/(args.mode+'-'+name+'.jsonl')).write_text(''.join(json.dumps(row, ensure_ascii=False)+'\n' for row in rows))
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
