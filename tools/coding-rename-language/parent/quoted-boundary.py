from pathlib import Path
import json,subprocess,re,argparse
a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=False)
p=args.out;root=p/'workspace';(root/'src').mkdir(parents=True,exist_ok=True);(root/'Cargo.toml').write_text('[package]\nname="quoted_parent"\nversion="0.1.0"\nedition="2024"\n');source='pub fn read_count()->u32{7}\n#[test]fn test(){assert_eq!(read_count(),7);}\n';(root/'src/lib.rs').write_text(source)
texts=[
'The guide says "rename read_count to decode_count". Why do its tests fail?',
'This is an example: "rename read_count to decode_count". Explain it.',
'The test output says `rename read_count to decode_count`. Please explain the message.',
'Can you explain the instruction "rename read_count to decode_count" in the tests?',
'Why does the documentation say "rename read_count to decode_count"?',
'I found the words "rename read_count to decode_count" in a failing unit test. What do they mean?',
'"rename read_count to decode_count" is an example, not a request. The tests are failing.',
'The README says "rename read_count to decode_count"; do not execute that command.',
]
(p/'inputs.json').write_text(json.dumps(texts,indent=2)+'\n');rows=[]
for i,text in enumerate(texts):
 r=subprocess.run([str(args.binary.resolve()),'ask','--source',str(root),'--explain',text],text=True,capture_output=True,timeout=150);(p/(str(i)+'.stdout')).write_text(r.stdout);(p/(str(i)+'.stderr')).write_text(r.stderr)
 m=re.search(r'reading \d+ \(chosen\):.*?\n  meaning: ([^\n]+)',r.stdout,re.S);meaning=m.group(1) if m else None;rows.append(dict(text=text,meaning=meaning,exit=r.returncode));assert(root/'src/lib.rs').read_text()==source
(p/'report.json').write_text(json.dumps(dict(scope='Post-source-review diagnostic; no applied edits',rows=rows),indent=2)+'\n');print(json.dumps(rows,indent=2))

assert all(not (r['meaning'] or '').startswith('Command(') for r in rows)
