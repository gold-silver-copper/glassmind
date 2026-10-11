from pathlib import Path
import hashlib,json,statistics,subprocess,time
r=Path('/home/gsc/.cache/glassmind/continuous-20261010/coordinator');a=r/'routeauthority-review/glassmind-integrated';b=r/'buffered-route-review/glassmind-integrated';model=Path('/home/gsc/.cache/glassmind/models/everyday/intent.json');rows=[]
for i in range(5):
 pair=[]
 for name,binary in [('baseline',a),('buffered',b)]:
  begin=time.monotonic();v=subprocess.run([str(binary),'talk','--json','--model',str(model)],input='/reset\nHello\n',capture_output=True,text=True,check=True,timeout=10);elapsed=time.monotonic()-begin;pair.append(v.stdout);rows.append(dict(repetition=i,variant=name,seconds=elapsed,reply_sha256=hashlib.sha256(v.stdout.encode()).hexdigest()))
 assert pair[0]==pair[1]
medians={k:statistics.median(x['seconds'] for x in rows if x['variant']==k) for k in ['baseline','buffered']}
report=dict(scope='Five interleaved fresh-process comparisons with identical model/input; concurrent development jobs active, no isolated-system claim',baseline_binary_sha256=hashlib.sha256(a.read_bytes()).hexdigest(),candidate_binary_sha256=hashlib.sha256(b.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),medians_seconds=medians,ratio=medians['baseline']/medians['buffered'],all_response_bytes_identical=True,rows=rows)
(r/'model-load-review/comparison.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
