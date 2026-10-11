#!/usr/bin/env python3
"""Public authored development probes; no benchmark or hidden data access."""
import argparse, hashlib, json, os, pathlib, subprocess
P = argparse.ArgumentParser()
P.add_argument('binary'); P.add_argument('output'); P.add_argument('--limit', type=int)
P.add_argument('--native', action='store_true')
P.add_argument('--cases', default=str(pathlib.Path(__file__).with_name('cases.tsv')))
a = P.parse_args(); binary = str(pathlib.Path(a.binary).resolve())
out = pathlib.Path(a.output).resolve(); out.mkdir(parents=True, exist_ok=True)
cases = [line.split('\t', 1) for line in pathlib.Path(a.cases).read_text().splitlines()]
if a.limit: cases = cases[:a.limit] + cases[24:27] + cases[36:39]
source = 'pub fn number()->u32 { 7u8 }\n#[test] fn value(){assert_eq!(number(),7);}\n'
fixed = source.replace('7u8', '7u32')
def run(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=180)
    return {'exit':r.returncode, 'stdout':r.stdout, 'stderr':r.stderr}
def native(work):
    return run(['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600','cargo','test','--offline','-j1','--','--test-threads','4'], work)
rows=[]
for i,(expected,text) in enumerate(cases):
    for mode in (['ask','apply','do'] if a.native else ['ask']):
        w = out / f'fixture-{i}-{mode}'; (w/'src').mkdir(parents=True, exist_ok=True)
        (w/'Cargo.toml').write_text('[package]\nname="repair_probe_' + hashlib.sha256(str(w).encode()).hexdigest()[:16] + '"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
        f=w/'src/lib.rs'; f.write_text(source)
        before = native(w) if a.native else None
        cmd=[binary,'do',str(w),'--explain',text] if mode=='do' else [binary,'ask','--source',str(w),'--json',text] + (['--apply'] if mode=='apply' else [])
        result=run(cmd); after=f.read_text(); check=native(w) if a.native else None
        if mode!='do':
            try: meaning=json.loads(result['stdout']).get('trace',{}).get('meaning','')
            except (ValueError, AttributeError): meaning=''
        else: meaning=result['stdout']+result['stderr']
        # Preserve raw outcomes; typed assertions also run inside the CLI unit test.
        mutation_ok = after == (fixed if expected.startswith('repair') and mode!='ask' else source)
        native_ok = not a.native or (before['exit']!=0 and (check['exit']==0)==(expected.startswith('repair') and mode!='ask'))
        rows.append(dict(expected=expected,text=text,mode=mode,result=result,meaning=meaning,
            source_before=source,source_after=after,before_sha256=hashlib.sha256(source.encode()).hexdigest(),
            after_sha256=hashlib.sha256(after.encode()).hexdigest(),native_before=before,native_after=check,
            mutation_ok=mutation_ok,native_ok=native_ok))
        (out/'results.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps({'cases':len(rows),'mutation_ok':sum(r['mutation_ok'] for r in rows),'native_ok':sum(r['native_ok'] for r in rows)}))
