#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
import hashlib,json,os,pathlib,subprocess
here=pathlib.Path(__file__).resolve().parent
run=pathlib.Path(os.environ['ROLES_RUN']).resolve();base=pathlib.Path(os.environ['ROLES_BASE']).resolve();source=pathlib.Path(os.environ['SQUAD_SOURCE']).resolve()
def item(p):return dict(path=str(p),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())
paths=list(base.glob('*'))+[source/n for n in ['train-v1.1.json','dev-v1.1.json','evaluate-v2.0.py','LICENSE','revision.txt']]
paths+=list(run.glob('*'))+[pathlib.Path(p) for p in ['crates/reading/src/qa.rs','crates/reading/src/bin/reading-qa.rs','crates/reading/src/probability.rs','tools/reading/qa/run.py']]
paths+=list(here.glob('*.py'))+list(here.glob('*.md'))+list(here.glob('*.sh'))+list(here.glob('*.patch'))+list((here/'reports').glob('*.json'))
paths=[p for p in paths if p.is_file() and p.name!='provenance.json']
report=dict(base_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),source_revision=(source/'revision.txt').read_text().strip(),data_license='SQuAD1.1 CC-BY-SA-4.0; repository/scorer MIT',source_url='https://github.com/rajpurkar/SQuAD-explorer/tree/eee5fdbf62f8613a7812b03419e6b29617b74fd1',new_external_data_bytes=0,dataset_bytes=35142551,files=[item(p) for p in paths])
(here/'reports/provenance.json').write_text(json.dumps(report,indent=2)+'\n')
