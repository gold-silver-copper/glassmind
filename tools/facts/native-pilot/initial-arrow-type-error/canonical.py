"""All-source canonical ID and numeric roundtrip prerequisite; no QA inputs."""
from pathlib import Path
import hashlib,json,time
import pyarrow as pa
import pyarrow.compute as pc
import pyarrow.parquet as pq
HERE=Path(__file__).resolve().parent
SOURCE=HERE.parent/'wikidata-snapshot'
started=time.monotonic()
manifest=json.loads((SOURCE/'pilot-manifest.json').read_text())
with (SOURCE/'pilot.sqlite').open('rb') as f: digest=hashlib.file_digest(f,'sha256').hexdigest()
assert digest=='c3f1197705f131e74f8f7b794d3f08eaef7d3b48e8a312170185e6728bff1313'

def validate(ids,pattern):
 assert ids.null_count==0
 assert pc.all(pc.match_substring_regex(ids,pattern)).as_py()
 numbers=pc.cast(pc.utf8_slice_codeunits(ids,1),pa.int64(),safe=True)
 restored=pc.binary_join_element_wise(pc.utf8_slice_codeunits(ids,0,1),pc.cast(numbers,pa.large_string()),'')
 assert pc.all(pc.equal(ids,restored)).as_py()
 return len(ids)

files=[];selected=0;names=0
for spec in manifest['files']:
 path=SOURCE/'raw'/spec['table']/spec['path']
 with path.open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==spec['sha256']
 is_claim=spec['table']=='claims';count=props=targets=0
 cols=['id','property','datatype','snaktype','rank','datavalue.id'] if is_claim else ['id']
 for batch in pq.ParquetFile(path).iter_batches(batch_size=65536,columns=cols,use_threads=False):
  ids=batch.column('id');count+=validate(ids,'^[QP][1-9][0-9]*$')
  if is_claim:
   props+=validate(batch.column('property'),'^P[1-9][0-9]*$')
   keep=pc.and_(pc.starts_with(ids,'Q'),pc.is_in(batch.column('rank'),value_set=pa.array(['normal','preferred'])))
   keep=pc.and_(keep,pc.equal(batch.column('snaktype'),'value'))
   keep=pc.and_(keep,pc.equal(batch.column('datatype'),'wikibase-item'))
   values=pc.filter(pc.struct_field(batch.column('datavalue'),'id'),keep)
   targets+=validate(values,'^Q[1-9][0-9]*$')
  else:names+=pc.sum(pc.starts_with(ids,'Q')).as_py()
 selected+=targets
 files.append({'table':spec['table'],'path':spec['path'],'subject_or_name_ids':count,'property_ids':props,'selected_target_ids':targets})
 print(json.dumps(files[-1]),flush=True)
assert selected==5677515 and names==84990012
report={'files':files,'selected_claims':selected,'selected_names':names,'database_sha256':digest,'seconds':round(time.monotonic()-started,3),'result':'all canonical and lossless signed-integer roundtrips'}
assert report['seconds']<180
(HERE/'canonical-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
