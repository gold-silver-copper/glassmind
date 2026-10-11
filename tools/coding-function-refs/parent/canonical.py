from pathlib import Path
import hashlib,json,shutil,subprocess
base=Path(__file__).parent;run=base.parent.parent/'functionrefs-run';out=base/'canonical';out.mkdir();root=out/'workspace';shutil.copytree(run/'dev-inputs/workspace',root)
manifest=json.loads((run/'dev-inputs/provenance.json').read_text())['files'];before={n:(root/n).read_bytes() for n in manifest}
for n,m in manifest.items():assert len(before[n])==m['bytes'] and hashlib.sha256(before[n]).hexdigest()==m['sha256']
expected={n:b.replace(b'parse_line',b'parse_entry') for n,b in before.items()};assert sum(b.count(b'parse_line') for b in before.values())==9
# All nine lexical occurrences were independently inspected: declaration/imports/calls/callbacks, none in literals/comments.
binary=run/'candidate';request='rename function parse_line to parse_entry'
def call(cmd,name):
 p=subprocess.run(cmd,cwd=root,text=True,capture_output=True,timeout=180);(out/(name+'.stdout')).write_text(p.stdout);(out/(name+'.stderr')).write_text(p.stderr);return p
native=call(['cargo','test','--offline','-j1','--','--test-threads','4'],'before-test');assert native.returncode==0
output=call(['cargo','run','--offline','-q','-j1'],'before-run');assert output.returncode==0
preview=call([str(binary),'ask','--source',str(root),'--json',request],'preview');j=json.loads(preview.stdout);assert j['response']['type']=='edit';edits=j['response']['changes'];assert len(edits)==9
for e in edits:
 line=before[e['file']].decode().splitlines()[e['line']-1];assert line[e['column']:e['column']+len(e['old'])]==e['old']=='parse_line';assert e['new']=='parse_entry'
assert all((root/n).read_bytes()==b for n,b in before.items())
apply=call([str(binary),'do',str(root),'--explain',request],'apply');assert apply.returncode==0;assert all((root/n).read_bytes()==b for n,b in expected.items())
newnative=call(['cargo','test','--offline','-j1','--','--test-threads','4'],'after-test');assert newnative.returncode==0
newoutput=call(['cargo','run','--offline','-q','-j1'],'after-run');assert newoutput.returncode==0 and newoutput.stdout==output.stdout
result=dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),supplied_files=5,supplied_bytes=sum(len(b) for b in before.values()),exact_identifier_edits=len(edits),original_tests_before_after=True,preview_read_only=True,binary_stdout_unchanged=True,stdout=output.stdout,all_source_bytes_exact=True)
(out/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
