#!/usr/bin/env python3
"""Frozen, exhaustive DEV accounting; inference files contain forms only.
No external scorer or evaluation definition is modified by this adapter.
"""
import argparse, collections, hashlib, json, pathlib, subprocess


def corpus(path):
    sentences = []
    counts = collections.Counter()
    for block in path.read_text().strip().split('\n\n'):
        rows = []
        sid = None
        for line in block.splitlines():
            if line.startswith('# sent_id = '):
                sid = line[len('# sent_id = '):]
            if line.startswith('#'):
                continue
            fields = line.split('\t')
            assert len(fields) == 10
            if '-' in fields[0]:
                counts['range_rows'] += 1
                continue
            if '.' in fields[0]:
                counts['decimal_rows'] += 1
                continue
            assert int(fields[0]) == len(rows) + 1
            rows.append(fields)
            counts['integer_rows'] += 1
        sentences.append({'id': sid, 'rows': rows})
    counts['sentences'] = len(sentences)
    return sentences, dict(counts)


def key(s, full=False):
    # Full integer-row annotation identity, excluding comment metadata.
    return tuple(tuple(r) if full else r[1] for r in s['rows'])


def valid(p, n):
    hs, ls = p.get('heads', []), p.get('deprels', [])
    if len(hs) != n or len(ls) != n or hs.count(0) != 1:
        return False
    for i, (h, l) in enumerate(zip(hs, ls), 1):
        if not isinstance(h, int) or not 0 <= h <= n or h == i or (h == 0) != (l == 'root'):
            return False
        seen, j = set(), i
        while j:
            if j in seen or not 1 <= j <= n:
                return False
            seen.add(j)
            j = hs[j - 1]
    return True


def nonprojective(hs):
    for i, h in enumerate(hs, 1):
        for k in range(min(i, h) + 1, max(i, h)):
            a = k
            for _ in hs:
                if a in (0, h):
                    break
                a = hs[a - 1]
            if a != h:
                return True
    return False


def write_json(path, obj):
    path.write_text(json.dumps(obj, indent=2, ensure_ascii=False) + '\n')


def score(sentences, predictions, train_forms, train_full, ledger):
    assert len(sentences) == len(predictions)
    totals = collections.defaultdict(collections.Counter)
    with ledger.open('w') as out:
        for s, p in zip(sentences, predictions):
            rows = s['rows']
            n = len(rows)
            forms = [r[1] for r in rows]
            emitted = p['status'] == 'predicted'
            if emitted:
                assert p['forms'] == forms
            tree_ok = emitted and valid(p, n)
            duplicate = key(s) in train_forms
            ann_duplicate = key(s, True) in train_full
            np = nonprojective([int(r[6]) for r in rows])
            record = {'id': s['id'], 'words': n, 'emitted': emitted,
                      'valid': tree_ok, 'train_forms_duplicate': duplicate,
                      'train_integer_rows_duplicate': ann_duplicate,
                      'gold_nonprojective': np, 'over128': n > 128}
            c = collections.Counter(sentences=1, words=n, emitted=emitted,
                                    valid=tree_ok, refused=not emitted,
                                    emitted_words=n if emitted else 0)
            errors = []
            for i, r in enumerate(rows):
                h, label = int(r[6]), r[7]
                bh, bl = (0, 'root') if i == 0 else (i, 'dep')
                u = tree_ok and p['heads'][i] == h
                l = u and p['deprels'][i] == label
                bu, bl_ok = bh == h, bh == h and bl == label
                c.update(uas=int(u), las=int(l), baseline_uas=int(bu), baseline_las=int(bl_ok))
                if r[3] != 'PUNCT':
                    c.update(nonpunct_words=1, nonpunct_uas=int(u), nonpunct_las=int(l),
                             baseline_nonpunct_uas=int(bu), baseline_nonpunct_las=int(bl_ok))
                if not l:
                    errors.append({'position': i+1, 'form': r[1], 'gold': [h, label],
                                   'predicted': [p['heads'][i], p['deprels'][i]] if emitted else None})
            record['counts'] = dict(c)
            record['errors'] = errors
            out.write(json.dumps(record, ensure_ascii=False) + '\n')
            for group in ['all', 'duplicate_forms' if duplicate else 'nonduplicate_forms',
                          'duplicate_annotations' if ann_duplicate else 'nonduplicate_annotations',
                          'nonprojective' if np else 'projective', 'over128' if n > 128 else 'within128']:
                totals[group].update(c)
    result = {}
    for name, c in totals.items():
        d = dict(c)
        for field in ['uas', 'las', 'baseline_uas', 'baseline_las']:
            d[field + '_percent'] = 100*c[field]/c['words'] if c['words'] else 0
        for field in ['nonpunct_uas', 'nonpunct_las', 'baseline_nonpunct_uas', 'baseline_nonpunct_las']:
            d[field + '_percent'] = 100*c[field]/c['nonpunct_words'] if c['nonpunct_words'] else 0
        result[name] = d
    c = totals['all']
    result['gates'] = {'las_at_least45': c['las'] / c['words'] >= .45,
                       'uas_at_least55': c['uas'] / c['words'] >= .55,
                       'beats_baseline_las': c['las'] > c['baseline_las'],
                       'beats_baseline_uas': c['uas'] > c['baseline_uas'],
                       'all_emitted_valid': c['emitted'] > 0 and c['emitted'] == c['valid']}
    result['tokenization'] = 'GOLD integer word forms, NOT raw-text accuracy'
    return result


def invoke(binary, model, inp, out, timing):
    subprocess.run(['systemd-run', '--user', '--scope',
                    '-pMemoryMax=8G', '-pMemorySwapMax=0', 'setsid', '--wait',
                    'timeout', '--kill-after=10s', '600', 'python3', 'tools/reading/syntax/measure.py', str(timing), str(binary), 'predict',
                    '--model', str(model), '--input', str(inp), '--output', str(out)], check=True)
    return [json.loads(s) for s in out.read_text().splitlines()]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--source', type=pathlib.Path, required=True)
    ap.add_argument('--artifacts', type=pathlib.Path, required=True)
    ap.add_argument('--binary', type=pathlib.Path, required=True)
    ap.add_argument('--model', type=pathlib.Path, required=True)
    ap.add_argument('--mode', choices=['sample', 'full', 'authored'], required=True)
    a = ap.parse_args()
    dst = a.artifacts
    if a.mode == 'authored':
        probes = [json.loads(l) for l in pathlib.Path('tools/reading/syntax/authored.jsonl').read_text().splitlines()]
        inp, out = dst/'authored-input.jsonl', dst/'authored-predictions.jsonl'
        inp.write_text(''.join(json.dumps({'text': p['text']}, ensure_ascii=False)+'\n' for p in probes))
        ps = invoke(a.binary, a.model, inp, out, dst/'authored-time.txt')
        passed, edges = 0, 0
        for probe, pred in zip(probes, ps):
            checks = []
            for d, h, label in probe.get('core', []):
                ok = pred['status'] == 'predicted' and pred['heads'][d-1] == h and pred['deprels'][d-1] == label
                checks.append({'dependent': d, 'head': h, 'label': label, 'correct': ok})
            probe['checks'], probe['prediction'] = checks, pred
            if probe['name'].startswith('active-'):
                passed += all(c['correct'] for c in checks)
                edges += sum(c['correct'] for c in checks)
        write_json(dst/'authored-report.json', {'active_sentences_correct': passed, 'active_sentences': 6,
                   'active_edges_correct': edges, 'active_edges': 12, 'gate': passed >= 4, 'probes': probes})
        print(json.dumps({'active_sentences_correct': passed, 'active_edges_correct': edges}))
        return
    train, tr_counts = corpus(a.source/'en_ewt-ud-train.conllu')
    dev, dev_counts = corpus(a.source/'en_ewt-ud-dev.conllu')
    train_forms, train_full = {key(s) for s in train}, {key(s, True) for s in train}
    ds = dev[:100] if a.mode == 'sample' else dev
    inp, out = dst/f'{a.mode}-input.jsonl', dst/f'{a.mode}-predictions.jsonl'
    inp.write_text(''.join(json.dumps({'forms': [r[1] for r in s['rows']]}, ensure_ascii=False)+'\n' for s in ds))
    ps = invoke(a.binary, a.model, inp, out, dst/f'{a.mode}-time.txt')
    r = score(ds, ps, train_forms, train_full, dst/f'{a.mode}-ledger.jsonl')
    r['source_row_counts'] = {'train': tr_counts, 'dev': dev_counts}
    r['input_sha256'] = hashlib.sha256(inp.read_bytes()).hexdigest()
    r['prediction_sha256'] = hashlib.sha256(out.read_bytes()).hexdigest()
    write_json(dst/f'{a.mode}-report.json', r)
    print(json.dumps({'mode': a.mode, 'all': r['all'], 'gates': r['gates']}, indent=2))


if __name__ == '__main__':
    main()
