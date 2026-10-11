#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent native check of the target-width witness and a failing control."""
import argparse, hashlib, json, os, shutil
from pathlib import Path
from run import cargo
p=argparse.ArgumentParser();p.add_argument('--dev-inputs',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
a.out.mkdir(parents=True,exist_ok=False)
rows=[]
for bits in (32,128):
    root=a.out/f'width-{bits}';shutil.copytree(a.dev_inputs/'workspace',root)
    text=(root/'src/lib.rs').read_text()
    text=text.replace('players[index]',f'players[{{ let _: [(); 1] = [(); (::core::primitive::usize::BITS >= {bits}) as usize]; (index) as usize }}]')
    (root/'src/lib.rs').write_text(text)
    result=cargo(root,dict(os.environ),'check')
    row=dict(required_bits=bits,source_sha256=hashlib.sha256(text.encode()).hexdigest(),bytes=len(text.encode()),result=result)
    row['passed']=(result['exit']==0 if bits==32 else result['exit']!=0 and 'E0308' in result['stderr'])
    rows.append(row)
(a.out/'results.json').write_text(json.dumps(rows,indent=2))
assert all(r['passed'] for r in rows)
print('native width witness and impossible-width control passed')
