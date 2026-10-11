"""Single frozen native fit and internal threshold calibration; no CAL inference."""
from collections import Counter
import hashlib,json,math,shutil,subprocess,time
from pathlib import Path
RUN=Path(__file__).resolve().parent
SOURCE=RUN.parent/'mention-candidates'
TRAIN=Path('/home/gsc/.cache/glassmind/data/simplequestions/train.tsv')
BINARY=RUN.parent/'disk-facts/target/release/examples/mention_context'
MARGINS=[0,0.1,0.25,0.5,0.75,1,1.5,2,3,4,8]
GAPS=[0,0.25,0.5,1,2]


def identity(path):
    with path.open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
    return dict(path=str(path),bytes=path.stat().st_size,sha256=digest)


def dump(name,value):
    with (RUN/name).open('x') as f:json.dump(value,f,indent=2);f.write('\n')


def execute(args,output):
    with (RUN/output).open('xb') as f,(RUN/(output+'.err')).open('xb') as err:
        subprocess.run([str(RUN/'candidate'),*args],stdout=f,stderr=err,check=True,timeout=900)


def main():
    started=time.monotonic();proof=json.loads((RUN/'pre-run.json').read_text());assert not (RUN/'candidate').exists()
    assert identity(Path(__file__))['sha256']==proof['scripts']['run.py']
    for path,expected in [(SOURCE/'candidates.jsonl','4fd5bd3906557789a6be39a894d38f645df1457738233a5d647cd81666061719'),(TRAIN,'25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b'),(SOURCE/'selection.json',proof['selection_sha256']),(BINARY,proof['binary_sha256'])]:
        assert identity(path)['sha256']==expected,path
    shutil.copyfile(BINARY,RUN/'candidate');(RUN/'candidate').chmod(0o555)
    selection=json.loads((SOURCE/'selection.json').read_text());indices=selection['fit_indices']
    source=[json.loads(line) for line in (SOURCE/'candidates.jsonl').read_text().splitlines()]
    assert len(source)==len(indices)==17465
    raw=TRAIN.read_text().splitlines();assert len(raw)==19481
    gold=[];folds=[];groups={};parity=[]
    for i,row in zip(indices,source,strict=True):
        annotation=raw[i].split('\t',3);assert row['question']==annotation[3] and i%10!=0
        normalized=row['normalized'];fold=int(hashlib.sha256(normalized.encode()).hexdigest(),16)%5
        assert groups.setdefault(normalized,fold)==fold
        folds.append(fold);gold.append(annotation[0])
        spans=row['candidates'];longest=max((len(s['text'].encode()) for s in spans),default=0)
        matches=[m for s in spans if len(s['text'].encode())==longest for m in s['matches']]
        ids={m['entity'] for m in matches};supported={m['entity'] for m in matches if m['case_supported']}
        baseline=next(iter(ids)) if len(ids)==1 and ids<=supported else None
        assert baseline==row['baseline_entity'] and row['parity'] is True
        parity.append(baseline)
    dump('split.json',dict(training_rows=sum(f!=0 for f in folds),validation_rows=sum(f==0 for f in folds),training_groups=sum(f!=0 for f in groups.values()),validation_groups=sum(f==0 for f in groups.values()),folds=folds,fit_indices=indices,all_baseline_links_reconstructed=True))
    execute(['features',str(SOURCE/'candidates.jsonl')],'features.jsonl')
    training_labels=Counter();span_count=0
    with (RUN/'train-examples.jsonl').open('x') as f:
        for index,line in enumerate((RUN/'features.jsonl').read_text().splitlines()):
            row=json.loads(line);assert row['row']==index and row['baseline_entity']==parity[index]
            span_count+=len(row['features'])
            if folds[index]==0:continue
            for span in row['features']:
                label='mention' if span['entity']==gold[index] else 'other';training_labels[label]+=1
                f.write(json.dumps(dict(text=span['text'],label=label))+'\n')
    assert index+1==len(source) and set(training_labels)=={'mention','other'}
    dump('training-input.json',dict(labels=dict(training_labels),training_examples=identity(RUN/'train-examples.jsonl'),features=identity(RUN/'features.jsonl'),total_eligible_span_occurrences=span_count))
    execute(['train',str(RUN/'train-examples.jsonl'),str(RUN/'model.json')],'training.json')
    assert (RUN/'model.json').stat().st_size<=8*1024**2
    execute(['score',str(RUN/'model.json'),str(RUN/'features.jsonl')],'scores.jsonl')
    dump('frozen-output.json',dict(model=identity(RUN/'model.json'),scores=identity(RUN/'scores.jsonl')))
    scores=[json.loads(line) for line in (RUN/'scores.jsonl').read_text().splitlines()];assert len(scores)==len(source)
    valid=[]
    for i,s in enumerate(scores):
        assert s['row']==i and s['baseline_entity']==parity[i]
        assert all(math.isfinite(v) for _,v in s['entity_scores'])
        if folds[i]==0:valid.append(i)
    grid=[]
    for margin in MARGINS:
        for gap in GAPS:
            before=Counter();after=Counter();transitions=Counter();changed=[]
            for i in valid:
                s=scores[i];old=parity[i];new=old;ranked=s['entity_scores']
                if ranked:
                    entity,value=ranked[0];second=ranked[1][1] if len(ranked)>1 else 0
                    if value>=margin and value-second>gap:new=entity
                a='right' if old==gold[i] else 'unlinked' if old is None else 'wrong'
                b='right' if new==gold[i] else 'unlinked' if new is None else 'wrong'
                before[a]+=1;after[b]+=1;transitions[f'{a}_to_{b}']+=1
                if new!=old:changed.append(dict(train_index=indices[i],before=old,after=new,gold=gold[i]))
            gain=after['right']-before['right'];newwrong=transitions['right_to_wrong']+transitions['unlinked_to_wrong'];lost=before['right']-transitions['right_to_right']
            grid.append(dict(margin=margin,gap=gap,baseline=dict(before),candidate=dict(after),transitions=dict(transitions),gain=gain,newly_wrong=newwrong,lost_correct=lost,passes=gain>=50 and newwrong==0 and lost==0,changed=changed))
    passing=[g for g in grid if g['passes']]
    selected=max(passing,key=lambda g:(g['gain'],g['margin'],g['gap'])) if passing else None
    dump('threshold-grid.json',grid)
    report=dict(validation_rows=len(valid),validation_groups=sum(f==0 for f in groups.values()),training_labels=dict(training_labels),model_bytes=(RUN/'model.json').stat().st_size,grid_points=len(grid),passing_grid_points=len(passing),gate_passed=selected is not None,selected={k:v for k,v in selected.items() if k!='changed'} if selected else None,maximum_gain_any_threshold=max(g['gain'] for g in grid),maximum_gain_with_zero_new_wrong=max((g['gain'] for g in grid if g['newly_wrong']==0 and g['lost_correct']==0),default=0),seconds=time.monotonic()-started,limitation='Grouped FIT-internal validation of entity linking only; no CAL/DEV/fullQA or runtime promotion.')
    dump('report.json',report);print(json.dumps(report),flush=True)


if __name__=='__main__':main()
