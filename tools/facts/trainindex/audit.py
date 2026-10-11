#!/usr/bin/env python3
"""Collect immutable run evidence; never changes index, model or score definitions."""
import argparse
from collections import Counter
from datetime import datetime
import hashlib
import json
from pathlib import Path
import subprocess


def ident(p):
    b=p.read_bytes()
    return dict(path=str(p.resolve()),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();a.output.mkdir(exist_ok=True)
    def read(name):return json.loads((a.run/name).read_bytes())
    sel=read('selection.json');fetch=read('fetch.json');build=read('rebuild.json');full=read('full/report.json');checks=read('bundle-checks.json')
    base=read('scope100/index.json')['entities'];expanded=read('scope1000/index.json')['entities']
    selected=[x['qid'] for x in sel['entities']]
    assert set(base)<=set(selected[:100]) and set(expanded)<=set(selected)
    assert all(expanded[q]==e for q,e in base.items())
    requests=fetch['requests']
    spacing=min((datetime.fromisoformat(y['started_utc'])-datetime.fromisoformat(x['started_utc'])).total_seconds() for x,y in zip(requests,requests[1:]))
    assert len(requests)<=80 and spacing>=2 and fetch['elapsed_seconds']<=600 and fetch['new_decompressed_bytes']<=80*1024*1024 and max(len(x['ids']) for x in requests)<=25
    assert max(Counter(tuple(x['ids']) for x in requests).values())<=2
    transitions=[json.loads(x) for x in (a.run/'full/transitions.jsonl').read_text().splitlines()]
    assert len(transitions)==2821
    hazards=[]
    for x in transitions:
        if x['transition']=='refused->wrong' or (x['transition'].startswith('right->') and x['transition']!='right->right'):
            entity=x['after']['entity'];relation=x['after']['relation']
            x['linked_gold_subject']=entity==x['gold_subject']
            x['analysis']='inverse local unique: does not prove global uniqueness or satisfy additional type restrictions' if relation and relation.startswith('R') else 'wrong entity alias link: factual claim belongs to a different subject'
            if relation and relation.startswith('R'):
                prop='P'+relation[1:]
                x['expanded_inverse_subjects']=[q for q,e in expanded.items() if entity in e['claims'].get(prop,[])]
            hazards.append(x)
    gained=[x for x in transitions if x['transition'] in ('refused->right','wrong->right')]
    lost=[x for x in transitions if x['transition'].startswith('right->') and x['transition']!='right->right']
    model=a.run/'bundle1000/relation.json'
    assert ident(model)['sha256']=='db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6'
    files=[p for p in a.run.rglob('*') if p.is_file() and 'relocated' not in p.parts]
    runtime_paths=['crates/facts/src/main.rs','crates/facts/src/lib.rs','crates/facts/examples/continuous_predictions.rs','crates/cli/src/talk_facts.rs','crates/cli/src/everyday.rs','tools/facts/chat/package.py','tools/facts/index.json','tools/facts/model-provenance.json','Cargo.lock']
    report=dict(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
                decision='REJECT expanded scope as default replacement: precision drop exceeds preregistered 0.02',
                selection={k:v for k,v in sel.items() if k!='entities'},selection_artifact=ident(a.run/'selection.json'),
                prefix100_qids_sha256=hashlib.sha256(('\n'.join(selected[:100])+'\n').encode()).hexdigest(),
                expanded1000_qids_sha256=hashlib.sha256(('\n'.join(selected)+'\n').encode()).hexdigest(),
                nested_entity_values_identical=True,minimum_request_spacing_seconds=spacing,
                full=full,sample=read('sample/report.json'),gained_correct=gained,earlier_correct_lost=lost,newly_wrong=hazards,
                transitions_full_artifact=ident(a.run/'full/transitions.jsonl'),bundle_checks=checks,
                source_hashes=[ident(Path(x)) for x in runtime_paths],adapter_hashes=[ident(p) for p in Path(__file__).parent.iterdir() if p.suffix in ('.py','.sh') or p.name=='PLAN.md'],
                artifacts=[ident(p) for p in sorted(files)],run_logs=[ident(a.run.parent/n) for n in ('build.log','python-tests.log','sample.log','full.log','bundle-checks.log','rebuild.log','fetch.log')],
                licenses={'SimpleQuestions':'CC BY 3.0; pinned '+sel['source_revision'], 'Wikidata structured labels aliases claims':'CC0-1.0','project':'MIT OR Apache-2.0'},
                numeric_wikidata_revisions='Unavailable: original wbgetentities props excluded info; exact raw SHA256 and client retrieval timestamps retained. Old cache retrieval times unavailable, only mtime. No revision numbers invented.',
                rejected_experiments=['Only preregistered 100->1000 expansion; rejected on full-dev precision. No dev-driven retry, threshold, claim filtering, entity reselection or default replacement.'])
    for name,doc in [('evidence.json',report),('fetch.json',fetch),('rebuild.json',build)]:
        (a.output/name).write_text(json.dumps(doc,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(decision=report['decision'],newly_wrong=len(hazards),earlier_correct_lost=len(lost),precision_baseline=full['scopes']['100']['precision'],precision_expanded=full['scopes']['1000']['precision'])))

if __name__=='__main__':main()
