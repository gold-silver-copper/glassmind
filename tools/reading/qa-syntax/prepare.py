#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Reconstruct the original split and verify identities before any fitting."""
import run as r,json
for x in json.loads((r.BASE/'manifest.json').read_text())['files']:
    p=r.BASE/x['path'];b=p.read_bytes();assert len(b)==x['bytes'] and r.q.digest(b)==x['sha256'],p
for x in json.loads((r.q.SOURCE/'provenance.json').read_text())['files']:
    p=r.q.SOURCE/x['file'];b=p.read_bytes();assert len(b)==x['bytes'] and r.q.digest(b)==x['sha256'],p
r.q.REPORT=r.RUN;r.q.prepare()
for name in ['fit.train.jsonl','fit.input.jsonl','cal.input.jsonl','cal.labels.json','selection.json']:assert (r.RUN/name).read_bytes()==(r.BASE/name).read_bytes(),name
print('11 baseline identities, 4 source identities, and 5 reconstructed files verified')
