#!/usr/bin/env python3
"""One frozen width-4 DEV run; unchanged independent full scoring definition."""
import argparse, collections, hashlib, importlib.util, json, pathlib, subprocess

ROOT = pathlib.Path(__file__).resolve().parents[3]
AUDIT_PATH = ROOT / 'tools/reading/syntax/parent/audit.py'
spec = importlib.util.spec_from_file_location('independent_audit', AUDIT_PATH)
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)

def digest(path):
    return {'bytes': path.stat().st_size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}

def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]

def predict(binary, model, source, output, beam=False):
    command = [str(binary), 'predict', '--model', str(model), '--input', str(source), '--output', str(output)]
    if beam:
        command.append('--beam')
    timing = pathlib.Path(str(output)+'.time.json')
    measured = ['timeout', '--kill-after=10s', '600', 'python3', str(ROOT/'tools/reading/syntax/measure.py'), str(timing), *command]
    subprocess.run(measured, check=True, timeout=620)
    result = json.loads(timing.read_text())
    return dict(wall_seconds=result['wall_seconds'], peak_rss_kib=result['max_rss_KiB'], command=command)

def grade(gold, predictions):
    result = collections.Counter(sentences=len(gold))
    assert len(gold) == len(predictions)
    for sentence, pred in zip(gold, predictions):
        emitted = pred.get('status') == 'predicted'
        result['refused'] += not emitted
        result['valid_trees'] += emitted and audit.is_tree(pred['heads'], pred['deprels'])
        if emitted:
            assert pred['forms'] == [x[0] for x in sentence]
        for i, (_, _, head, label) in enumerate(sentence):
            result['words'] += 1
            uas = emitted and pred['heads'][i] == head
            result['uas_correct'] += uas
            result['las_correct'] += uas and pred['deprels'][i] == label
    return dict(result)

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--run', type=pathlib.Path, required=True)
    p.add_argument('--source', type=pathlib.Path, required=True)
    args = p.parse_args()
    run = args.run.resolve(); out = run/'candidate'; base = run/'baseline'
    binary = out/'reading-syntax'; model = base/'model.bin'
    assert not (out/'FREEZE.json').exists(), 'Run is one-shot; never overwrite a candidate DEV run'
    own = pathlib.Path(__file__).resolve().parent
    for name, identity in json.loads((base/'manifest.json').read_text()).items():
        assert digest(base/name) == identity, f'baseline identity mismatch: {name}'
    provenance = json.loads((ROOT/'tools/reading/syntax/source-provenance.json').read_text())
    for identity in provenance['files']:
        if identity['name'] in ['en_ewt-ud-train.conllu','en_ewt-ud-dev.conllu']:
            assert digest(args.source/identity['name']) == {k:identity[k] for k in ['bytes','sha256']}
    frozen = [ROOT/'crates/reading/src/syntax.rs', ROOT/'crates/reading/src/bin/reading-syntax.rs',
              AUDIT_PATH, ROOT/'tools/reading/syntax/measure.py', binary, *own.glob('*.py'), *own.glob('*.rs'), own/'reproduce.sh', own/'PLAN.md',
              own/'BOUND-NOTE.md', own/'contrasts.jsonl', *[base/k for k in json.loads((base/'manifest.json').read_text())],
              args.source/'en_ewt-ud-train.conllu', args.source/'en_ewt-ud-dev.conllu']
    write(out/'FREEZE.json', {str(f): digest(f) for f in frozen})
    # Existing adapter/model contracts run unmodified.
    (out/'model.bin').symlink_to(model)
    subprocess.run(['python3', str(ROOT/'tools/reading/syntax/contracts.py'), str(out)], check=True)
    timings = {}
    for name, source in [('forms',base/'forms.jsonl'), ('raw',base/'raw-inputs.jsonl')]:
        target = out/f'default-{name}.jsonl'
        timings[f'default_{name}'] = predict(binary, model, source, target)
        assert target.read_bytes() == (base/f'{name}-predictions.jsonl').read_bytes(), f'default {name} drift'
    # Measure the original binary under the same tool and input for slowdown.
    timings['baseline_forms'] = predict(base/'reading-syntax', model, base/'forms.jsonl', out/'baseline-replay.jsonl')
    assert (out/'baseline-replay.jsonl').read_bytes() == (base/'forms-predictions.jsonl').read_bytes()
    # Authored probes never carry labels into runtime.
    probes = rows(own/'contrasts.jsonl')
    (out/'probe-inputs.jsonl').write_text(''.join(json.dumps({'forms':c['forms']})+'\n' for c in probes))
    for mode, use_beam in [('greedy',False),('beam',True)]:
        timings[f'probes_{mode}'] = predict(binary,model,out/'probe-inputs.jsonl',out/f'probes-{mode}.jsonl',use_beam)
    probe_results=[]
    for case, old, new in zip(probes,rows(out/'probes-greedy.jsonl'),rows(out/'probes-beam.jsonl')):
        edges=[]
        for dep,head,label in case['core_edges']:
            before=(old['heads'][dep-1],old['deprels'][dep-1])==(head,label)
            after=(new['heads'][dep-1],new['deprels'][dep-1])==(head,label)
            edges.append(dict(edge=[dep,head,label],baseline_correct=before,beam_correct=after))
        probe_results.append(dict(case=case,edges=edges))
    write(out/'probe-results.json', probe_results)
    # Beam contract refusals differ only by explicit search provenance.
    predict(binary,model,out/'contracts-input.jsonl',out/'beam-contracts.jsonl',True)
    old_contracts=rows(out/'contracts-output.jsonl'); new_contracts=rows(out/'beam-contracts.jsonl')
    for old,new in zip(old_contracts,new_contracts):
        assert new.pop('search_mode') == 'beam_4_sum_stable'
        assert old['status'] == new['status']
        if old['status']=='refused': assert old==new
    # Fixed first100 followed by one full2001 run; never tune after these outputs.
    gold = audit.corpus(args.source/'en_ewt-ud-dev.conllu')
    baseline = rows(base/'forms-predictions.jsonl')
    (out/'sample-inputs.jsonl').write_text(''.join((base/'forms.jsonl').read_text().splitlines(keepends=True)[:100]))
    timings['beam_sample'] = predict(binary, model, out/'sample-inputs.jsonl', out/'sample-predictions.jsonl', True)
    write(out/'sample-score.json', dict(baseline=grade(gold[:100],baseline[:100]),beam=grade(gold[:100],rows(out/'sample-predictions.jsonl'))))
    timings['beam_full'] = predict(binary, model, base/'forms.jsonl', out/'beam-predictions.jsonl', True)
    candidate = rows(out/'beam-predictions.jsonl')
    assert candidate[:100] == rows(out/'sample-predictions.jsonl')
    for old,new in zip(baseline,candidate):
        assert new['search_mode']=='beam_4_sum_stable'
        assert old['upos']==new['upos'] and old['forms']==new['forms']
    for name,pred in [('baseline',base/'forms-predictions.jsonl'),('beam',out/'beam-predictions.jsonl')]:
        subprocess.run(['python3',str(AUDIT_PATH),'--source',str(args.source),'--predictions',str(pred),'--out',str(out/f'{name}-audit')],check=True)
    seen={tuple(x[0] for x in sentence) for sentence in audit.corpus(args.source/'en_ewt-ud-train.conllu')}
    counts=collections.defaultdict(collections.Counter); edge_ledger=[]; sentence_ledger=[]
    for sid,(sentence,old,new) in enumerate(zip(gold,baseline,candidate)):
        duplicate=tuple(x[0] for x in sentence) in seen
        group='duplicate_forms' if duplicate else 'nonduplicate_forms'
        sentence_counts=collections.Counter()
        for i,(form,_,head,label) in enumerate(sentence):
            oh,ol=old['heads'][i],old['deprels'][i]; nh,nl=new['heads'][i],new['deprels'][i]
            transitions={}
            for metric,before,after in [('uas',oh==head,nh==head),('las',(oh,ol)==(head,label),(nh,nl)==(head,label))]:
                transition=('correct' if before else 'wrong')+'->'+('correct' if after else 'wrong')
                transitions[metric]=transition
                for g in ['all',group]: counts[g][metric+'_'+transition]+=1
                sentence_counts[metric+'_'+transition]+=1
            if (oh,ol)!=(nh,nl):
                edge_ledger.append(dict(sentence=sid,token=i+1,form=form,duplicate=duplicate,gold=[head,label],baseline=[oh,ol],beam=[nh,nl],**transitions))
        before_exact=all((old['heads'][i],old['deprels'][i])==(x[2],x[3]) for i,x in enumerate(sentence))
        after_exact=all((new['heads'][i],new['deprels'][i])==(x[2],x[3]) for i,x in enumerate(sentence))
        transition=('correct' if before_exact else 'wrong')+'->'+('correct' if after_exact else 'wrong')
        for g in ['all',group]: counts[g]['sentence_'+transition]+=1
        sentence_ledger.append(dict(sentence=sid,duplicate=duplicate,baseline_exact=before_exact,beam_exact=after_exact,changed=old['heads']!=new['heads'] or old['deprels']!=new['deprels'],transitions=dict(sentence_counts)))
    for name,ledger in [('edge-transitions',edge_ledger),('sentence-transitions',sentence_ledger)]:
        (out/f'{name}.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in ledger))
    full=grade(gold,candidate)
    lost=sum(e['baseline_correct'] and not e['beam_correct'] for c in probe_results for e in c['edges'])
    summary=dict(baseline=grade(gold,baseline),beam=full,transitions={k:dict(v) for k,v in counts.items()},
        authored=dict(cases=len(probes),edges=72,baseline_correct=sum(e['baseline_correct'] for c in probe_results for e in c['edges']),beam_correct=sum(e['beam_correct'] for c in probe_results for e in c['edges']),lost_correct=lost),
        gates=dict(las=full['las_correct']>=19673,uas=full['uas_correct']>=20540,trees=full['valid_trees']==2001,no_refusals=full['refused']==0,authored_preservation=lost==0,default_forms_byte_identical=True,default_raw_byte_identical=True,independent_parent_contrasts='pending'),
        timings=timings,slowdown=timings['beam_full']['wall_seconds']/timings['baseline_forms']['wall_seconds'])
    write(out/'summary.json',summary)
    print(json.dumps(summary,indent=2))

if __name__=='__main__': main()
