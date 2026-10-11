from pathlib import Path
import argparse,hashlib,json,re,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();base=Path(__file__).parent;raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];cases=json.loads(raw);a.out.mkdir(parents=True,exist_ok=False)
source='pub fn number()->u32 { 7u8 }\n#[test] fn value(){assert_eq!(number(),7);}\n';expected=source.replace('7u8','7u32');rows=[]
for c in cases:
 for mode in ['ask','do']:
  case=a.out/(c['name']+'-'+mode);root=case/'workspace';(root/'src').mkdir(parents=True);(root/'Cargo.toml').write_text('[package]\nname="speech_parent"\nversion="0.1.0"\nedition="2024"\n');(root/'src/lib.rs').write_text(source)
  def run(cmd,label):
   p=subprocess.run(cmd,cwd=root,capture_output=True,text=True,timeout=150);(case/(label+'.stdout')).write_text(p.stdout);(case/(label+'.stderr')).write_text(p.stderr);return p
  preview=run([str(a.binary.resolve()),'ask','--source',str(root.resolve()),'--explain',c['text']],'preview');assert(root/'src/lib.rs').read_text()==source
  cmd=[str(a.binary.resolve()),'ask','--source',str(root.resolve()),'--apply','--explain'] if mode=='ask' else [str(a.binary.resolve()),'do',str(root.resolve()),'--explain']
  response=run(cmd+[c['text']],'action');after=(root/'src/lib.rs').read_text();changed=after!=source;native=None
  if changed:native=run(['cargo','test','--offline','-j1','--','--test-threads','4'],'after-test').returncode
  meanings=re.findall(r'  meaning: ([^\n]+)',preview.stdout);has_command=any(m.startswith('Command(') for m in meanings)
  rows.append(dict(name=c['name'],expected=c['expected'],check=c.get('check'),mode=mode,preview_meanings=meanings,preview_mutating=has_command,changed=changed,exact_repair=after==expected,native_exit=native,action_exit=response.returncode))
  print(c['name'],mode,'changed',changed,'preview_command',has_command,flush=True)
  (a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),source_fixture=source,cases=rows),indent=2)+'\n')
