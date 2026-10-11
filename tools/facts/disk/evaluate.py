"""Frozen same-data parity and command/RSS checks; questions only enter inference."""
import hashlib,json,pathlib,subprocess,time,os
ROOT=pathlib.Path('/home/gsc/code/glassmind')
WORK=pathlib.Path('/home/gsc/.cache/glassmind/continuous-20261010')
HERE=pathlib.Path(__file__).resolve().parent
BIN=HERE/'target/release'
ART=WORK/'linkquality-run/artifacts'
BUNDLE=WORK/'trainindex-run/tmp/trainindex/bundle1000'

def identity(path):
 with path.open('rb') as stream: digest=hashlib.file_digest(stream,'sha256').hexdigest()
 return dict(path=str(path),bytes=path.stat().st_size,sha256=digest)

def checked(args,tag,stdin=None,timeout=550):
 input_path=HERE/f'{tag}.stdin'
 if stdin is not None:input_path.write_bytes(stdin)
 started=time.monotonic()
 with (input_path if stdin is not None else pathlib.Path('/dev/null')).open('rb') as inp,(HERE/f'{tag}.out').open('wb') as out,(HERE/f'{tag}.err').open('wb') as err:
  child=subprocess.Popen(list(map(str,args)),stdin=inp,stdout=out,stderr=err)
  while True:
   pid,status,usage=os.wait4(child.pid,os.WNOHANG)
   if pid:break
   if time.monotonic()-started>timeout:
    child.kill();pid,status,usage=os.wait4(child.pid,0)
    child.returncode=os.waitstatus_to_exitcode(status)
    raise subprocess.TimeoutExpired(args,timeout)
   time.sleep(.05)
  child.returncode=os.waitstatus_to_exitcode(status)
  (HERE/f'{tag}.time.json').write_text(json.dumps({'wall_seconds':round(time.monotonic()-started,4),'max_rss_kib':usage.ru_maxrss})+'\n')
  if child.returncode:raise subprocess.CalledProcessError(child.returncode,args)
 return (HERE/f'{tag}.out').read_bytes()

started=time.monotonic()
manifest={r['path']:r for r in json.loads((ROOT/'tools/facts/linkquality/artifacts.json').read_text())['artifacts']}
inputs=[]
for name in ['train.questions','valid.questions','new999-train-base.jsonl','new999-valid-frozen.jsonl']:
 p=ART/name;r=identity(p);assert r==manifest[str(p)];inputs.append(r)
for name,expected in [('index.json','142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791'),('relation.json','db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6')]:
 r=identity(BUNDLE/name);assert r['sha256']==expected;inputs.append(r)
(HERE/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
index=HERE/'native999.sqlite'
checked([BIN/'glassmind-facts','index-disk','--index',BUNDLE/'index.json','--database',index],'import')
report={'scope':'same-data regression equivalence; not unseen QA accuracy','splits':{},'inputs':inputs}
for split,n in [('train',19481),('valid',2821)]:
 questions=(ART/f'{split}.questions').read_bytes();assert len(questions.splitlines())==n
 raw=checked([BIN/'examples/disk_parity',BUNDLE/'relation.json',BUNDLE/'index.json',index],f'{split}-parity',questions)
 rows=[json.loads(r) for r in raw.splitlines()];assert len(rows)==n
 baseline=ART/('new999-train-base.jsonl' if split=='train' else 'new999-valid-frozen.jsonl')
 old=[json.loads(r) for r in baseline.read_text().splitlines()];assert len(old)==n
 assert [r['answer'] for r in rows]==old, 'retained baseline changed'
 # Exercise actual user-facing streaming command, including full sets.
 for sets in [False,True]:
  tag=f'{split}-cli'+('-set' if sets else '')
  args=[BIN/'glassmind-facts','predict-disk','--model',BUNDLE/'relation.json','--database',index]
  if sets:args.append('--set')
  output=checked(args,tag,questions)
  assert [json.loads(r) for r in output.splitlines()]==[r['set' if sets else 'answer'] for r in rows]
 report['splits'][split]={'rows':n,'answer_parity':n,'set_parity':n,'retained_baseline_parity':n,'cli_answer_parity':n,'cli_set_parity':n,'parity_resources':json.loads((HERE/f'{split}-parity.time.json').read_text()),'cli_resources':json.loads((HERE/f'{split}-cli.time.json').read_text())}
 print(json.dumps({split:report['splits'][split]}),flush=True)
# Fixed first1,000 TRAIN questions: compare memory/startup+total elapsed with the
# preserved original baseline executable, not an instrumented combined process.
questions=b'\n'.join((ART/'train.questions').read_bytes().splitlines()[:1000])+b'\n'
base=WORK/'linkquality-run/baseline/predictions'
assert identity(base)['sha256']=='1e5a4af21b05024c85c6e3173f782cd6d62c3a5ed27259fe642aeb30c56eb433'
a=checked([base,BUNDLE/'relation.json',BUNDLE/'index.json'],'baseline-1000',questions)
b=checked([BIN/'glassmind-facts','predict-disk','--model',BUNDLE/'relation.json','--database',index],'disk-1000',questions)
assert [json.loads(r) for r in a.splitlines()]==[json.loads(r) for r in b.splitlines()]
report['performance_1000']={k:json.loads((HERE/f'{k}-1000.time.json').read_text()) for k in ['baseline','disk']}
report['seconds']=round(time.monotonic()-started,3);assert report['seconds']<600
report['database']=identity(index)
report['binaries']=[identity(BIN/'glassmind-facts'),identity(BIN/'examples/disk_parity'),identity(base)]
(HERE/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
