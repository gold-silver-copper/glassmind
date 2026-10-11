#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Record frozen artifact identities and aggregate baseline transitions."""
import collections, hashlib, json, os, pathlib, subprocess
import run
ROOT=run.HERE.parents[2]
def identity(p):
    b=p.read_bytes(); return dict(bytes=len(b),sha256=hashlib.sha256(b).hexdigest())

def main():
    rs=sorted(run.rows(run.load('dev-v1.1.json')['data']),key=run.order)
    outputs={name:[json.loads(l) for l in (run.ART/f'full.{name}.outputs.jsonl').read_text().splitlines()] for name in ('baseline','candidate')}
    assert len(rs)==len(outputs['baseline'])==len(outputs['candidate'])==10570
    transitions=collections.Counter(); raw_transitions=collections.Counter()
    for r,b,c in zip(rs,outputs['baseline'],outputs['candidate']):
        def exact(text): return max(run.SCORE['compute_exact'](a['text'],text) for a in r['answers'])
        def status(p): return 'refused' if p['status']=='refused' else 'right' if exact(p['answer']) else 'wrong'
        transitions[status(b)+' -> '+status(c)]+=1
        raw_transitions[str(exact(run.proposal(r,b)))+' -> '+str(exact(run.proposal(r,c)))]+=1
    model=json.loads((run.ART/'model.json').read_text())
    run.write(run.REPORT/'audit.json',dict(accepted_transitions=dict(sorted(transitions.items())),raw_exact_transitions=dict(sorted(raw_transitions.items())),nonzero_weights=sum(x!=0 for x in model['weights']),dense_f32_weight_bytes=len(model['weights'])*4,fit_representable=model['representable_rows'],fit_unrepresentable=model['fit_rows']-model['representable_rows'],no_dev_tuning=True))
    before=json.loads((run.HERE/'reports/pre_style_fix_hashes.json').read_text())
    assert all(identity(run.ART/name)['sha256']==sha for name,sha in before.items()), 'final rebuild changed predictions or model'
    source_files=['crates/reading/src/lib.rs','crates/reading/src/qa.rs','crates/reading/src/bin/reading-qa.rs','crates/cli/src/reading.rs','crates/cli/src/main.rs','crates/cli/Cargo.toml','Cargo.lock','tools/reading/qa/run.py','tools/reading/qa/attest.py','tools/reading/qa/reproduce.sh','tools/reading/qa/check-main.py','tools/reading/qa/PLAN.md']
    source={f:identity(ROOT/f) for f in source_files}
    unchanged={}
    for f in ['crates/reading/src/main.rs','crates/reading/src/probability.rs','crates/reading/src/bin/reading-probability.rs','crates/reading/Cargo.toml','Cargo.toml','Cargo.lock']:
        original=subprocess.check_output(['git','show','HEAD:'+f],cwd=ROOT)
        assert (ROOT/f).read_bytes()==original
        unchanged[f]=identity(ROOT/f)
    binary=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release/reading-qa'
    main_binary=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release/glassmind'
    assert (run.ART/'main.full.outputs.jsonl').read_bytes()==(run.ART/'full.candidate.outputs.jsonl').read_bytes()
    record=dict(base_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),source_files=source,source_manifest_sha256=hashlib.sha256(json.dumps(source,sort_keys=True).encode()).hexdigest(),binary=dict(path=str(binary),**identity(binary)),rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),dataset=run.load('provenance.json'),source_files_verified={f:identity(run.SOURCE/f) for f in ['train-v1.1.json','dev-v1.1.json','evaluate-v2.0.py','LICENSE']},artifacts={p.name:identity(p) for p in sorted(run.ART.iterdir()) if p.is_file()},unchanged_paths=unchanged,final_rebuild_predictions_byte_identical=True,code_license='MIT',derived_weights_reports_license='CC-BY-SA-4.0',new_download_bytes=0)
    for f in record['dataset']['files']:
        assert record['source_files_verified'][f['file']]==dict(bytes=f['bytes'],sha256=f['sha256'])
    record['main_binary']=dict(path=str(main_binary),**identity(main_binary))
    record['main_cli_full_predictions_byte_identical']=True
    run.write(run.REPORT/'provenance.json',record)
    print(json.dumps(dict(source_manifest_sha256=record['source_manifest_sha256'],binary=record['binary'],model=record['artifacts']['model.json'],accepted_transitions=transitions),indent=2))
if __name__=='__main__': main()
