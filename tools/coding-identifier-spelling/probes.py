"""Independent identifier preservation probes, frozen before parent candidate replay.
A mutation must preserve the user's full identifier; a refusal may not mutate.
Underscores before call parentheses are semantically significant in Rust.
"""
from pathlib import Path
import argparse,hashlib,json,os,subprocess
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(exist_ok=False)
cases=[('caller','renamed_()', 'renamed_'),('caller','called__()', 'called__'),('caller','saved_()', 'saved_'),('caller','`renamed_()`','renamed_'),('caller','renamed_', 'renamed_'),('tick','renamed_()', 'renamed_'),('caller','renamed','renamed'),('caller','naïve','naïve')]
rows=[]
env=dict(os.environ);env['GLASSMIND_SCRATCH']=str(a.out/'scratch');env['CARGO_BUILD_JOBS']='1';env['RUST_TEST_THREADS']='4'
for i,(source,destination,expected) in enumerate(cases):
 root=a.out/str(i);(root/'src').mkdir(parents=True);(root/'tests').mkdir()
 src=f'pub fn {source}() -> u32 {{ 7 }}\npub fn entry() -> u32 {{ {source}() }}\n'
 (root/'src/lib.rs').write_text(src);(root/'Cargo.toml').write_text('[package]\nname="identifier_provenance"\nversion="0.0.0"\nedition="2021"\n[workspace]\n');(root/'tests/check.rs').write_text('#[test]\nfn same() { assert_eq!(identifier_provenance::entry(),7); }\n')
 def snapshot():return {str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest() for f in root.rglob('*') if f.is_file()}
 before=snapshot();request=f'rename function {source} to {destination}';v=subprocess.run([str(a.binary.resolve()),'do',str(root),request],capture_output=True,text=True,env=env,timeout=120);after=snapshot();actual=(root/'src/lib.rs').read_text();mutated=after!=before
 wanted=src.replace(f'fn {source}(',f'fn {expected}(').replace(f'{{ {source}() }}',f'{{ {expected}() }}')
 safe=not mutated or (actual==wanted and sorted(k for k in before.keys()|after.keys() if before.get(k)!=after.get(k))==['src/lib.rs'])
 rows.append(dict(request=request,expected_identifier=expected,mutated=mutated,safe=safe,stdout=v.stdout,stderr=v.stderr,actual=actual,before=before,after=after))
report=dict(scope='Authored identifier-source preservation probes; refusal is allowed but mutation must preserve complete spelling',binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows,total=len(rows),safe=sum(x['safe'] for x in rows),mutations=sum(x['mutated'] for x in rows))
(a.out/'report.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
