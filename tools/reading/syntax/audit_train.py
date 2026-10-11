#!/usr/bin/env python3
"""TRAIN-only counts and skip ledger; never reads DEV or official test."""
import collections
import json
import pathlib
import sys
from evaluate import corpus, nonprojective

source, out = map(pathlib.Path, sys.argv[1:])
ss, rows = corpus(source/'en_ewt-ud-train.conllu')
c = collections.Counter()
with (out/'train-skips.jsonl').open('w') as f:
    for s in ss:
        rs = s['rows']
        n = len(rs)
        forms = [r[1] for r in rs]
        np = nonprojective([int(r[6]) for r in rs])
        long = n > 128
        large = sum(len(w.encode()) for w in forms) > 16384
        invalid_form = any(not w or any(ch.isspace() or ord(ch) < 32 or 127 <= ord(ch) <= 159 for ch in w) for w in forms)
        reasons = [name for name, yes in [('over128', long), ('over16KiB', large), ('invalid_form', invalid_form), ('nonprojective', np)] if yes]
        if reasons:
            f.write(json.dumps({'id': s['id'], 'words': n, 'reasons': reasons})+'\n')
        c['all_sentences'] += 1
        c['all_words'] += n
        if not (long or large or invalid_form):
            c['tagger_sentences'] += 1
            c['tagger_words'] += n
            if not np:
                c['parser_sentences'] += 1
                c['parser_words'] += n
        for reason in reasons:
            c[reason + '_sentences'] += 1
            c[reason + '_words'] += n
(out/'train-audit.json').write_text(json.dumps({'counts':dict(c), 'rows':rows}, indent=2)+'\n')
