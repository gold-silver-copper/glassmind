from pathlib import Path
import argparse,hashlib,json,re,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();r=Path('/home/gsc/.cache/glassmind/continuous-20261010');ind=r/'coordinator/repairspeech-independent';cases=json.loads((ind/'cases.json').read_text());old={x['name']:x for x in json.loads((ind/'revision/report.json').read_text())['cases'] if x['mode']=='ask'};a.out.mkdir(exist_ok=False);rows=[]
for c in cases:
 w=a.out/c['name'];(w/'src').mkdir(parents=True);(w/'Cargo.toml').write_text('[package]\nname="release_contract"\nversion="0.1.0"\nedition="2024"\n');source='pub fn number()->u32 { 7u8 }\n#[test] fn value(){assert_eq!(number(),7);}\n';(w/'src/lib.rs').write_text(source)
 v=subprocess.run([str(a.binary.resolve()),'ask','--source',str(w.resolve()),'--explain',c['text']],capture_output=True,text=True,timeout=150);(w/'stdout').write_text(v.stdout);(w/'stderr').write_text(v.stderr);meanings=re.findall(r'  meaning: ([^\n]+)',v.stdout)
 assert meanings==old[c['name']]['preview_meanings'],(c['name'],meanings,old[c['name']]['preview_meanings']);assert (w/'src/lib.rs').read_text()==source
 rows.append(dict(name=c['name'],meaning=meanings,source_unchanged=True,debug_contract_identical=True));(a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows),indent=2)+'\n')
print('All30 integrated release contracts match independently tested debug meanings; no edits')
