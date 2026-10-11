"""Unmodified main conversation binary byte replay; no route integration."""
import hashlib,json,subprocess
from pathlib import Path
W=Path('/home/gsc/.cache/glassmind/continuous-20261010');R=W/'timedmatch-run';O=R/'evidence/main';O.mkdir(exist_ok=True)
bundle=O/'bundle';bundle.mkdir(exist_ok=True)
for name,src in [('index.json',W/'factmatch-source/index.json'),('relation.json',W/'factmatch-source/relation.json'),('temporal.json',W/'timedmatch-source/temporal.json')]:
    target=bundle/name
    if not target.exists():target.symlink_to(src)
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
(bundle/'manifest.json').write_text(json.dumps(dict(version=1,index_sha256=h(bundle/'index.json'),relation_model_sha256=h(bundle/'relation.json'),temporal_sha256=h(bundle/'temporal.json'),source='Existing cached Wikidata responses',license='CC0; questions CC BY 3.0',scope='Existing 500 validation-subject-seeded snapshot')))
rows=(W/'factmatch-source/valid.tsv').read_text().splitlines();report=[]
for n in [200,len(rows)]:
    inp=''.join(row.split('\t',3)[3]+'\n' for row in rows[:n]).encode()
    for mode in ['default','facts']:
        outputs=[]
        for name,binary in [('baseline',R/'baseline/glassmind'),('current',R/'target/release/glassmind')]:
            cmd=[str(binary),'talk','--json']+(['--facts-dir',str(bundle)] if mode=='facts' else [])
            p=subprocess.run(cmd,input=inp,capture_output=True,check=True)
            (O/f'{name}-{mode}-{n}.jsonl').write_bytes(p.stdout);outputs.append(p.stdout)
        assert outputs[0]==outputs[1],(mode,n)
        report.append(dict(mode=mode,questions=n,bytes=len(outputs[0]),sha256=hashlib.sha256(outputs[0]).hexdigest(),exact=True))
(O/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report))
