#!/usr/bin/env python3
"""Evaluate frozen TRAIN scope with unchanged scorer and question-only inference."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess


def run(args, text=None):
    return subprocess.run(list(map(str,args)), input=text, text=True,
                          capture_output=True, timeout=550, check=True).stdout

def outcome(row, pred):
    if pred['answers'] == [row[2]]:
        return 'right'
    return 'refused' if pred['refusal'] else 'wrong'

def identity(path):
    data = path.read_bytes()
    return dict(path=str(path.resolve()), bytes=len(data), sha256=hashlib.sha256(data).hexdigest())

def write(path, obj):
    path.write_text(json.dumps(obj, indent=2, sort_keys=True)+'\n')

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run', type=Path, required=True)
    p.add_argument('--target', type=Path, required=True)
    p.add_argument('--valid', type=Path, required=True)
    p.add_argument('--limit', type=int)
    a = p.parse_args()
    assert a.valid.name == 'valid.tsv'
    rows = [x.split('\t',3) for x in a.valid.read_text().splitlines()][:a.limit]
    assert len(rows) == (a.limit or 2821)
    tag = 'sample' if a.limit else 'full'
    out = a.run/tag
    out.mkdir(exist_ok=True)
    questions = ''.join(r[3]+'\n' for r in rows)
    (out/'questions.txt').write_text(questions)
    predictions, summary = {}, {}
    binaries = [a.target/'release/glassmind-facts', a.target/'release/examples/continuous_predictions', a.target/'release/glassmind']
    for size in (100,1000):
        bundle = a.run/f'bundle{size}'
        # Existing authoritative scorer, unchanged; annotations are consumed by
        # scorer only. Separate inference subprocess receives just questions.
        cmd = [binaries[0], 'score', '--input', a.valid, '--index', bundle/'index.json', '--model', bundle/'relation.json']
        if a.limit:
            cmd += ['--limit', a.limit]
        score = json.loads(run(cmd))
        write(out/f'score{size}.json',score)
        raw = run([binaries[1],bundle/'relation.json',bundle/'index.json'],questions)
        (out/f'standalone{size}.jsonl').write_text(raw)
        preds = [json.loads(x) for x in raw.splitlines()]
        predictions[size] = preds
        talk = run([binaries[2],'talk','--json','--facts-dir',bundle], ''.join('/reset\n'+r[3]+'\n' for r in rows))
        (out/f'talk{size}.jsonl').write_text(talk)
        all_replies = [json.loads(x) for x in talk.splitlines()]
        assert len(all_replies) == 2*len(rows)
        assert all(x['kind']=='reset' for x in all_replies[::2])
        counts = Counter(outcome(row,pred) for row,pred in zip(rows,preds,strict=True))
        assert all(score[k] == counts[k] for k in ('right','refused','wrong'))
        changes = [dict(row=i+1,standalone=pred,talk=reply) for i,(pred,reply) in enumerate(zip(preds,all_replies[1::2],strict=True)) if pred['answers'] != reply.get('answers',[])]
        summary[str(size)] = dict(score=score, precision=score['right']/(score['right']+score['wrong']) if score['right']+score['wrong'] else None, answer_id_changes=changes, assets=[identity(bundle/n) for n in ('manifest.json','index.json','relation.json','labels.json')])
        print(tag, size, json.dumps(summary[str(size)]['score']), flush=True)
    transitions = []
    for i,(row,before,after) in enumerate(zip(rows,predictions[100],predictions[1000],strict=True)):
        transitions.append(dict(row=i+1, question=row[3], gold_subject=row[0],gold_relation=row[1],gold_object=row[2], before=before, after=after, transition=outcome(row,before)+'->'+outcome(row,after)))
    (out/'transitions.jsonl').write_text(''.join(json.dumps(x)+'\n' for x in transitions))
    matrix = dict(Counter(x['transition'] for x in transitions))
    hazards = [x for x in transitions if (x['transition'].endswith('->wrong') and not x['transition'].startswith('wrong->')) or (x['transition'].startswith('right->') and x['transition'] != 'right->right')]
    write(out/'hazards.json',hazards)
    report = dict(series='TRAIN-prefix90-fit subject-frequency finite Wikidata scope v1',total=len(rows),scopes=summary,transitions=matrix,hazards=len(hazards),binaries=[identity(x) for x in binaries],validation=identity(a.valid))
    report['metric_acceptance'] = summary['1000']['score']['right'] > summary['100']['score']['right'] and summary['100']['precision'] is not None and summary['1000']['precision'] is not None and summary['1000']['precision'] >= summary['100']['precision']-.02
    report['exact_id_equivalence'] = not any(x['answer_id_changes'] for x in summary.values())
    write(out/'report.json',report)
    print(json.dumps(dict(transitions=matrix,metric_acceptance=report['metric_acceptance'],exact_id_equivalence=report['exact_id_equivalence'])),flush=True)

if __name__ == '__main__':
    main()
