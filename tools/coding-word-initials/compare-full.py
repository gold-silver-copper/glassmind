"""Compare aggregate public DEV outcomes; never read hidden task material."""
import argparse,collections,hashlib,json
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--before',type=Path,required=True);p.add_argument('--after',type=Path,required=True);p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);args=p.parse_args()
load=lambda f:[json.loads(line) for line in f.read_text().splitlines()]
oldrows,newrows=load(args.before),load(args.after);key=lambda r:(r['task'],r['style']);old={key(r):r for r in oldrows};new={key(r):r for r in newrows}
assert len(oldrows)==len(newrows)==len(old)==len(new)==765 and old.keys()==new.keys()
changes=[dict(task=k[0],style=k[1],before=r['outcome'],after=new[k]['outcome']) for k,r in old.items() if r['outcome']!=new[k]['outcome']]
counts=collections.Counter(r['outcome'] for r in newrows);retained=sum(r['outcome']=='pass' and new[k]['outcome']=='pass' for k,r in old.items());regressions=[c for c in changes if c['before']=='pass' or c['after']=='fail'];sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
report=dict(scope='Full765 public DEV, unchanged runner and denominator; no hidden task contents inspected',total=765,outcomes=dict(counts),all_previous_passes_retained=retained,regressions=regressions,outcome_changes=changes,per_kind={kind:dict(collections.Counter(r['outcome'] for r in newrows if r['kind']==kind)) for kind in sorted({r['kind'] for r in newrows})},baseline_result_sha256=sha(args.before),candidate_result_sha256=sha(args.after),binary_sha256=sha(args.binary),runner_sha256=sha(Path('tools/rustbench-dev.sh')))
assert not args.out.exists();args.out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));assert retained==220 and not regressions and counts.get('fail',0)==0 and counts['pass']>=221
