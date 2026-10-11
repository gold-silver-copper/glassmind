from pathlib import Path
import collections,hashlib,json,statistics
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator');before=r/'identifierroles-full/results.jsonl';after=r/'fieldowners-full/results.jsonl'
load=lambda p:[json.loads(l) for l in p.read_text().splitlines()]
a=load(before);b=load(after);assert len(a)==len(b)==765
key=lambda x:(x['task'],x['style']);old={key(x):x for x in a};new={key(x):x for x in b};assert len(old)==len(new)==765 and old.keys()==new.keys()
changes=[dict(task=k[0],style=k[1],before=x['outcome'],after=new[k]['outcome']) for k,x in old.items() if x['outcome']!=new[k]['outcome']]
counts=collections.Counter(x['outcome'] for x in b);retained=sum(x['outcome']=='pass' and new[k]['outcome']=='pass' for k,x in old.items());perkind={kind:dict(collections.Counter(x['outcome'] for x in b if x['kind']==kind)) for kind in sorted({x['kind'] for x in b})}
binary=r/'fieldowners-review/glassmind-integrated'
report=dict(scope='Full fixed765 Rust coding dev; no held-out evaluation',commit='4a06d788e923ca29e40ddc1e6f5d0e8d50f20015',binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),total=765,outcomes=dict(counts),all_previous_passes_retained=retained,outcome_changes=changes,per_kind=perkind,baseline_result_sha256=hashlib.sha256(before.read_bytes()).hexdigest(),candidate_result_sha256=hashlib.sha256(after.read_bytes()).hexdigest(),timing=dict(sum_agent_seconds=sum(x['agent_secs'] for x in b),median_agent_seconds=statistics.median(x['agent_secs'] for x in b)),runner='Unchanged tools/rustbench-dev.sh; persistent TMPDIR and explicit GLASSMIND_SCRATCH; memory8GiB/0swap, setsid --wait, bounded timeouts10s shutdown grace')
(r/'fieldowners-review/full-comparison.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));assert retained==174 and counts.get('fail',0)==0 and counts['pass']>=174
