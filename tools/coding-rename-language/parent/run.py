from pathlib import Path
import argparse,hashlib,json,re,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();base=Path(__file__).parent
raw=(base/'cases.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==json.loads((base/'freeze.json').read_text())['sha256'];data=json.loads(raw);a.out.mkdir(parents=True,exist_ok=False);results=[]
for c in data['cases']:
 root=a.out/c['name'];(root/'src').mkdir(parents=True);(root/'Cargo.toml').write_text('[package]\nname="language_parent"\nversion="0.1.0"\nedition="2024"\n');(root/'src/lib.rs').write_text(data['source'])
 def run(cmd,label):
  r=subprocess.run(cmd,cwd=root,capture_output=True,text=True,timeout=150);(root/(label+'.stdout')).write_text(r.stdout);(root/(label+'.stderr')).write_text(r.stderr);return r
 def meaning(s):
  m=re.search(r'reading \d+ \(chosen\):.*?\n  meaning: ([^\n]+)',s,re.S);return m.group(1) if m else None
 ask=run([str(a.binary.resolve()),'ask','--source','workspace='+str(root.resolve()),'--explain',c['text']],'ask');assert(root/'src/lib.rs').read_text()==data['source']
 action=run([str(a.binary.resolve()),'do',str(root.resolve()),'--explain',c['text']],'do');after=(root/'src/lib.rs').read_text();changed=after!=data['source'];expected=data['source'].replace('fn read_count','fn decode_count').replace('{ read_count(20)','{ decode_count(20)');exact=after==expected if changed else True
 native_exit=None
 if changed:
  native=run(['cargo','test','--offline','-j1','--','--test-threads','4'],'after-test');native_exit=native.returncode
 row=dict(name=c['name'],expected=c['expected'],required=c['required'],ask_meaning=meaning(ask.stdout),do_meaning=meaning(action.stdout),changed=changed,exact=exact,native_exit=native_exit,ask_exit=ask.returncode,do_exit=action.returncode)
 results.append(row);print(json.dumps(row),flush=True)
 (a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=results),indent=2)+'\n')
