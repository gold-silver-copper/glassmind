#!/usr/bin/env python3
"""Build a fixed coding-only iteration sample, preserving whole conversations.

Full track development runs remain the batch check. This selects from training
only and prints aggregate counts and hashes, never example text.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path

KINDS={'build_status','fix_build','test_status','fix_tests','rb_fix_error','rb_fix_test','rb_edit','rb_implement'}


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--train',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--per-kind',type=int,default=16)
    args=p.parse_args()
    if args.per_kind<1:p.error('per-kind must be positive')
    if args.train.resolve()==args.out.resolve():p.error('output must differ from training directory')
    load=lambda name:[json.loads(l) for l in (args.train/name).read_text().splitlines() if l.strip()]
    labels={r['turn_id']:r for r in load('labels.jsonl')}
    groups=defaultdict(list)
    for row in load('inputs.jsonl'):
        linked=[labels[t['id']] for t in row['turns']]
        if any(label['split']!='train' for label in linked):continue
        kind=next((label['kind'] for label in linked if label['kind'] in KINDS),None)
        if kind is not None:
            key=hashlib.sha256(('glassmind-fast-v1:'+row['turns'][0]['id']).encode()).hexdigest()
            groups[kind].append((key,row))
    selected=[row for kind in sorted(groups) for _,row in sorted(groups[kind],key=lambda r:r[0])[:args.per_kind]]
    chosen=[labels[t['id']] for row in selected for t in row['turns']]
    args.out.mkdir(parents=True,exist_ok=True)
    for name,rows in [('inputs.jsonl',selected),('labels.jsonl',chosen)]:
        (args.out/name).write_text(''.join(json.dumps(r)+'\n' for r in rows))
    manifest={'seed':'glassmind-fast-v1','conversations':len(selected),'turns':len(chosen),
              'per_kind':dict(Counter(r['kind'] for r in chosen)),
              'source':str(args.train.resolve()),'per_kind_limit':args.per_kind,
              'sha256':{name:hashlib.sha256((args.out/name).read_bytes()).hexdigest() for name in ['inputs.jsonl','labels.jsonl']}}
    (args.out/'sample.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps(manifest,indent=2))

if __name__=='__main__':main()
