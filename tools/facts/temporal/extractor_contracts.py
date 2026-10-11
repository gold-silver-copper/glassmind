"""Small authored extractor integrity tests; uses no network or historical fixtures."""
import hashlib,json,subprocess,sys
from pathlib import Path
here=Path(__file__).resolve().parent;run=Path(sys.argv[1]);root=run/'tmp/extractor-contracts';root.mkdir(parents=True,exist_ok=True)
source=root/'source';source.mkdir(exist_ok=True)
index=here/'index.json';ib=index.read_bytes();ih=hashlib.sha256(ib).hexdigest()
(root/'index-manifest.json').write_text(json.dumps({'index.json':{'bytes':len(ib),'sha256':ih}}))
cases=json.loads((here/'contracts.json').read_text());statement=next(c for c in cases if c['name']=='extra qualifier')['statements'][0]
raw=json.dumps({'entities':{'Q1':{'claims':{'P108':[statement]}}}}).encode();filename='0000000000000000.json'
def prepare(raw):
 (source/filename).write_bytes(raw)
 (source/'manifest.json').write_text(json.dumps({'index_sha256':ih,'files':[{'file':filename,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}]}))
def call(output):
 return subprocess.run([sys.executable,str(here/'extract.py'),'--source',str(source),'--index',str(index),'--index-manifest',str(root/'index-manifest.json'),'--output',str(root/output)],capture_output=True,timeout=30)
prepare(raw);assert call('first.json').returncode==0;assert call('second.json').returncode==0
assert (root/'first.json').read_bytes()==(root/'second.json').read_bytes()
record=json.loads((root/'first.json').read_bytes())['records'][0];assert record['statement']==statement and 'P1480' in record['statement']['qualifiers']
(source/filename).write_bytes(raw+b' ');badraw=call('badraw.json');assert badraw.returncode!=0
prepare(raw);m=json.loads((source/'manifest.json').read_text());m['index_sha256']='0'*64;(source/'manifest.json').write_text(json.dumps(m));badhash=call('badhash.json');assert badhash.returncode!=0
# Duplicated raw keys must never be erased by JSON last-key-wins during extraction.
prepare(raw.replace(b'"entities":',b'"entities":{},"entities":',1));duplicate=call('duplicate.json');assert duplicate.returncode!=0
(run/'reports/extractor-contracts.json').write_text(json.dumps(dict(deterministic=True,full_extra_qualifier_preserved=True,raw_hash_mismatch_rejected=True,index_hash_mismatch_rejected=True,duplicate_raw_keys_rejected=True,expected_failure_stderr=[r.stderr.decode() for r in [badraw,badhash,duplicate]]),indent=2)+'\n')
print('5 extractor integrity contracts passed')
