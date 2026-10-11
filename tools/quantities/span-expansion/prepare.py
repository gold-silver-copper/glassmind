"""Select frozen replication TRAIN-FIT rows and verify their existing native numeric spans."""
import hashlib,json,re,subprocess,sys
from pathlib import Path
source=Path('/home/gsc/.cache/glassmind/continuous-20261010/groundedsteps-source')
old=source.parent/'groundedsteps-run/frozen-candidate'
out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=json.loads((source/'manifest.json').read_text())
assert manifest['official_split']=='train' and manifest['rows']==7473
verified=[]
for row in manifest['files']+manifest['reused_native_files']:
 p=source/row['file'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'];verified.append(dict(path=str(p),sha256=row['sha256'],bytes=row['bytes']))
archive=json.loads((old/'artifact-manifest.json').read_text())
for row in archive['files']:
 p=old/row['file'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256']
rows=[]
for line in (old/'full/records.jsonl').open():
 r=json.loads(line)
 if r['subset']=='TRAIN-FIT' and r['reasons']==['ambiguous_leaf']:rows.append(r)
rows.sort(key=lambda r:(r['group_sha256'],r['row_id']))
chosen=[];groups=set()
for r in rows:
 if r['group_sha256'] in groups:continue
 groups.add(r['group_sha256']);chosen.append(r)
 if len(chosen)==160:break
assert len(chosen)==160
pilot=json.loads((source.parent/"coordinator/span-references/source/batch.json").read_text())["rows"]
assert [r["row_id"] for r in chosen[:32]]==[r["row_id"] for r in pilot]
chosen=chosen[32:]
assert len(chosen)==128
byid={r['row_id']:r for r in chosen};questions={};bindings={}
for i,line in enumerate((source/'train.jsonl').open()):
 if i not in byid:continue
 raw=json.loads(line);r=byid[i]
 assert hashlib.sha256(raw['question'].encode()).hexdigest()==r['question_sha256']
 assert hashlib.sha256(raw['answer'].encode()).hexdigest()==r['answer_sha256']
 normalized=re.sub(r'[0-9]+','#',' '.join(raw['question'].lower().split()))
 assert hashlib.sha256(normalized.encode()).hexdigest()==r['group_sha256']
 assert int(r['group_sha256'],16)%5!=0
 questions[i]=raw
assert i+1==7473
for i,line in enumerate((old/'full/question-only-binding.outputs.jsonl').open()):
 if i in byid:bindings[i]=json.loads(line)
requests=[dict(question=questions[r['row_id']]['question'],program=None) for r in chosen]
completed=subprocess.run([str(source/'equation_executor')],input=''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in requests),text=True,capture_output=True,timeout=30,check=True)
actual=[json.loads(line) for line in completed.stdout.splitlines()];assert len(actual)==128
batch=[]
for r,native in zip(chosen,actual,strict=True):
 i=r['row_id'];assert native==bindings[i] and native['status']=='bound'
 raw=questions[i]
 for n in native['numbers']:
  assert raw['question'].encode()[n['start']:n['end']].decode()==n['literal']
 batch.append(dict(id=f'fit-{i}',row_id=i,group_sha256=r['group_sha256'],question=raw['question'],answer=raw['answer'],numbers=native['numbers'],annotations=r['annotations'],expression_nodes=r['expression_nodes'],leaves=r['leaves'],final_answer=r['final_answer']))
constants=['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']
(out/'batch.json').write_text(json.dumps(dict(schema=1,semantics='TRAIN annotation inputs; no inference evaluation',constants=constants,rows=batch),ensure_ascii=False,indent=2)+'\n')
(out/'native-requests.jsonl').write_text(''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in requests))
(out/'native-outputs.jsonl').write_text(completed.stdout)
report=dict(source=manifest,source_manifest_sha256=sha(source/'manifest.json'),archive_manifest_sha256=sha(old/'artifact-manifest.json'),original_archive_files_verified=len(archive['files']),source_files_verified=verified,selection='next128 distinct TRAIN-FIT groups after pilot first32, by(group_sha256,row_id), sole reason ambiguous_leaf',rows=128,row_ids=[r['row_id'] for r in chosen],parent_reference_row_ids=[r['row_id'] for r in [chosen[j] for j in (0,1,32,33,64,65,96,97)]],batch_sha256=sha(out/'batch.json'),all_native_bindings_reproduced=True,baseline_resolved_in_selected_batch=0,original_converter_global_accepted=354,original_converter_global_total=7473,teacher_calls=0)
(out/'prepare-report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ('source','source_files_verified')}))

for index in range(4):
 dest=out/f"batch-{index+1:02d}";dest.mkdir()
 data=dict(schema=1,semantics="TRAIN annotation replication; no inference evaluation",constants=constants,rows=batch[index*32:(index+1)*32])
 (dest/"batch.json").write_text(json.dumps(data,ensure_ascii=False,indent=2)+"\n")
report["batches"]=[dict(batch=i+1,rows=32,sha256=sha(out/f"batch-{i+1:02d}/batch.json")) for i in range(4)]
(out/"prepare-report.json").write_text(json.dumps(report,indent=2)+"\n")
