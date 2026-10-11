from pathlib import Path
import argparse,json,subprocess,hashlib
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False);root=a.out/'workspace';(root/'src').mkdir(parents=True)
(root/'Cargo.toml').write_text('[package]\nname="context_parent"\nversion="0.1.0"\nedition="2024"\n')
source='pub fn read_count(x:i32)->i32{x*2+3}\npub fn run()->i32{read_count(20)}\n#[test]fn behavior(){assert_eq!(run(),43);}\n';(root/'src/lib.rs').write_text(source);session=a.out/'session.json'
def ask(text,label,apply=False):
 cmd=[str(a.binary.resolve()),'ask','--source','workspace='+str(root.resolve()),'--session',str(session.resolve()),'--json'];cmd+=['--apply'] if apply else [];r=subprocess.run(cmd+[text],text=True,capture_output=True,timeout=150);(a.out/(label+'.stdout')).write_text(r.stdout);(a.out/(label+'.stderr')).write_text(r.stderr);return r,json.loads(r.stdout)
r,topic=ask('where is function read_count?','topic');assert r.returncode==0 and topic['response']['type']=='items';initial_session=session.read_bytes()
r,missing=ask('rename to decode_count','missing',True);unsafe=(root/'src/lib.rs').read_text()!=source
(root/'src/lib.rs').write_text(source);session.write_bytes(initial_session)
r,explicit=ask('rename it to decode_count','explicit',True);after=(root/'src/lib.rs').read_text();expected=source.replace('read_count','decode_count');assert r.returncode==0 and after==expected
native=subprocess.run(['cargo','test','--offline','-j1','--','--test-threads','4'],cwd=root,text=True,capture_output=True,timeout=120);(a.out/'native.stdout').write_text(native.stdout);(a.out/'native.stderr').write_text(native.stderr);assert native.returncode==0
result=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),omitted_target_mutated=unsafe,omitted_target_response=missing['response'],explicit_pronoun_exact_edit=True,explicit_pronoun_native_pass=True)
(a.out/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
