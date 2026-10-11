#!/usr/bin/env python3
"""Frozen candidate evaluation; official scorer and joint audit are imported unchanged."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
from collections import Counter
import experiment as ex

ROOT=ex.OWN.parents[2]
TARGET=Path(os.environ['CARGO_TARGET_DIR'])/'release'

def invoke(args, text=None, cwd=None):
    result=subprocess.run(list(map(str,args)),input=text,text=True,capture_output=True,timeout=550,cwd=cwd)
    if result.returncode:raise RuntimeError(result.stderr)
    return result.stdout


def freeze():
    candidate=ex.RUN/'candidate-index.json'
    labels=json.loads((ex.BASE/'scope1000/labels.json').read_bytes())
    labels['index_sha256']=ex.sha(candidate.read_bytes())
    ex.write(ex.RUN/'candidate-labels.json',labels)
    invoke(['python3',ROOT/'tools/facts/chat/package.py','--index',candidate,
        '--model',ex.BASE/'bundle1000/relation.json','--labels',ex.RUN/'candidate-labels.json',
        '--source','Independent primary WDQS statements; TRAIN-only top100 inverse pairs',
        '--license','CC0-1.0','--scope','Prospective incoming-edge experiment; partial/truncated sets; no global uniqueness',
        '--output',ex.RUN/'candidate-bundle'])
    paths=[*sorted(ex.OWN.glob('*.py')),ex.OWN/'PLAN.md',ex.OWN/'selection.json',
        ROOT/'tools/facts/joint-audit.py',ROOT/'tools/facts/chat/package.py',
        *sorted((ROOT/'crates/facts/src').glob('*.rs')),
        ROOT/'crates/facts/examples/continuous_predictions.rs',
        TARGET/'glassmind-facts',TARGET/'glassmind',TARGET/'examples/continuous_predictions',
        *sorted((ex.RUN/'candidate-bundle').glob('*')),ex.RUN/'fetch.json',ex.RUN/'merge.json']
    doc=dict(frozen_utc=ex.utc(),source='b569ac0eb0bab5a8b9d39cda904c816e77c08e63',artifacts=[ex.identity(p) for p in paths])
    with (ex.RUN/'candidate-freeze.json').open('x') as f:json.dump(doc,f,indent=2)
    print('candidate and scripts frozen before DEV',flush=True)


def status(row,pred,joint=False):
    if pred['refusal'] is not None:return 'refused'
    return 'right' if pred['answers']==[row[2]] and (not joint or (pred['entity']==row[0] and pred['relation']==row[1])) else 'wrong'


def evaluate(limit):
    frozen=json.loads((ex.RUN/'candidate-freeze.json').read_bytes())
    for r in frozen['artifacts']:ex.checked(Path(r['path']),r)
    out=ex.RUN/('sample' if limit else 'full');out.mkdir()
    valid=ex.DATA/'valid.tsv'
    rows=[l.split('\t',3) for l in valid.read_text().splitlines()][:limit]
    assert len(rows)==(limit or 2821)
    questions=''.join(r[3]+'\n' for r in rows);(out/'questions.txt').write_text(questions)
    spec=importlib.util.spec_from_file_location('joint_audit',ROOT/'tools/facts/joint-audit.py')
    audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
    results={};predictions={};disagreements=[]
    for name,bundle in [('baseline',ex.BASE/'bundle1000'),('candidate',ex.RUN/'candidate-bundle')]:
        cmd=[TARGET/'glassmind-facts','score','--input',valid,'--index',bundle/'index.json','--model',bundle/'relation.json']
        if limit:cmd+=['--limit',limit]
        score=json.loads(invoke(cmd));ex.write(out/f'{name}-score.json',score)
        raw=invoke([TARGET/'examples/continuous_predictions',bundle/'relation.json',bundle/'index.json'],questions)
        (out/f'{name}-standalone.jsonl').write_text(raw);preds=[json.loads(l) for l in raw.splitlines()]
        predictions[name]=preds
        joint=audit.audit(rows,preds)
        assert joint['total']==score['total'] and all(joint['answer_only'][k]==score[k] for k in ('right','refused','wrong'))
        ex.write(out/f'{name}-joint.json',joint)
        talk=invoke([TARGET/'glassmind','talk','--json','--facts-dir',bundle],''.join('/reset\n'+r[3]+'\n' for r in rows))
        (out/f'{name}-talk.jsonl').write_text(talk);replies=[json.loads(l) for l in talk.splitlines()]
        assert len(replies)==2*len(rows) and all(r['kind']=='reset' for r in replies[::2])
        for i,(p,r) in enumerate(zip(preds,replies[1::2],strict=True)):
            if p['answers']!=r.get('answers',[]):disagreements.append(dict(scope=name,row=i+1,standalone=p,talk=r))
        results[name]=dict(answer=joint['answer_only'],joint=joint['joint_entity_relation_answer'])
        print(out.name,name,json.dumps(results[name]),flush=True)
    transitions=[];links=[]
    for i,(row,b,a) in enumerate(zip(rows,predictions['baseline'],predictions['candidate'],strict=True)):
        transitions.append(dict(row=i+1,gold=dict(entity=row[0],relation=row[1],answer=row[2]),question=row[3],before=b,after=a,
            answer=status(row,b)+'->'+status(row,a),joint=status(row,b,True)+'->'+status(row,a,True)))
        if b['entity']!=a['entity']:links.append(dict(row=i+1,before=b['entity'],after=a['entity']))
    ex.write(out/'transitions.json',transitions)
    ex.write(out/'disagreements.json',disagreements)
    ex.write(out/'alias-link-changes.json',links)
    hazards=[t for t in transitions if any(t[k].endswith('->wrong') and not t[k].startswith('wrong->') for k in ('answer','joint')) or any(t[k].startswith('right->') and t[k]!='right->right' for k in ('answer','joint'))]
    ex.write(out/'hazards.json',hazards)
    matrices={k:dict(Counter(t[k] for t in transitions)) for k in ('answer','joint')}
    def precision(counts):return counts['right']/(counts['right']+counts['wrong']) if counts['right']+counts['wrong'] else 0
    gates=dict(joint_at_least_7=results['candidate']['joint']['right']>=7,
        retained_old_joint_correct=all(t['joint']=='right->right' for t in transitions if t['joint'].startswith('right->')),
        no_new_joint_wrong=all(t['joint'] not in ('right->wrong','refused->wrong') for t in transitions),
        precision_not_worse=all(precision(results['candidate'][k])>=precision(results['baseline'][k]) for k in ('answer','joint')),
        exact_alias_links=not links,standalone_talk_ids_equal=not disagreements)
    # Relocate candidate and replay the exact question-only stream from another cwd.
    relocated=ex.RUN/'relocated'/out.name/'bundle';relocated.parent.mkdir(parents=True)
    shutil.copytree(ex.RUN/'candidate-bundle',relocated)
    moved=invoke([TARGET/'glassmind','talk','--json','--facts-dir','bundle'],''.join('/reset\n'+r[3]+'\n' for r in rows),cwd=relocated.parent)
    (out/'relocated-talk.jsonl').write_text(moved)
    gates['relocated_byte_identical']=moved==(out/'candidate-talk.jsonl').read_text()
    report=dict(total=len(rows),results=results,transitions=matrices,gates=gates,accepted=all(gates.values()),hazards=len(hazards),valid=ex.identity(valid))
    ex.write(out/'report.json',report);print(json.dumps(report),flush=True)
    if not limit:
        assert results['baseline']['answer']==dict(right=47,refused=2712,wrong=62)
        assert results['baseline']['joint']==dict(right=2,refused=2712,wrong=107)


if __name__=='__main__':
    import sys
    {'freeze':freeze,'sample':lambda:evaluate(200),'full':lambda:evaluate(None)}[sys.argv[1]]()
