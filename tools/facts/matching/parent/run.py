import argparse,json,hashlib,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();root=Path(__file__).parent;a.out.mkdir();index=root/'index.json';before=index.read_bytes();cases=json.loads((root/'cases.json').read_text());records=[]
for c in cases:
 if '\0' in c['question']:
  records.append({'name':c['name'],'library_only':True,'reason':'NUL cannot cross argv; separate native library gate required'});continue
 proc=subprocess.run([str(a.binary),'match','--index',str(index),c['question']],capture_output=True,text=True,timeout=15);actual=None;err=None
 try:
  assert proc.returncode==0,proc.stderr;actual=json.loads(proc.stdout);assert actual['index_sha256']==hashlib.sha256(before).hexdigest();kind=c['expected_kind'];status=actual['status']
  if kind=='unknown_or_refused':assert status in ['unknown','refused'],actual
  elif kind=='matches_or_explicit_unsupported' and status=='refused':assert actual['reason']=='unsupported_grammar'
  else:
   assert status=='matched',actual
   for k in ['total_snapshot_matches','returned','truncated']:assert actual[k]==c[k],(k,actual[k],c[k])
   assert [m['subject'] for m in actual['matches']]==c['subjects'];assert [[x['property'],x['object']] for x in actual['constraints']]==c['constraints'];assert actual['scope']=='local_snapshot' and 'temporal' in actual['semantics']
   for m in actual['matches']:
    expected=[{'subject':m['subject'],'property':p,'object':o,'source_url':f"https://www.wikidata.org/wiki/{m['subject']}#{p}"} for p,o in c['constraints']];assert m['witnesses']==expected
  assert index.read_bytes()==before
 except Exception as e:err=str(e)
 records.append({'name':c['name'],'passed':err is None,'error':err,'actual':actual,'stderr':proc.stderr})
(a.out/'results.json').write_text(json.dumps(records,ensure_ascii=False,indent=2)+'\n');summary={'native_cli_cases':sum('passed'in r for r in records),'passed':sum(r.get('passed',False) for r in records),'failures':[r['name'] for r in records if r.get('error') is not None],'library_pending':[r['name'] for r in records if r.get('library_only')],'binary_sha256':hashlib.sha256(a.binary.read_bytes()).hexdigest(),'cases_sha256':hashlib.sha256((root/'cases.json').read_bytes()).hexdigest()};(a.out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2));assert not summary['failures']
