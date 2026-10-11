"""One bounded FIT-only extraction; no annotations passed to native extraction."""
from collections import Counter
import hashlib,json,shutil,subprocess,time
from pathlib import Path

RUN=Path(__file__).resolve().parent
WORK=RUN.parent.parent
TRAIN=Path('/home/gsc/.cache/glassmind/data/simplequestions/train.tsv')
GRAPH=RUN.parent/'full-claims/native.sqlite'
BINARY=RUN.parent/'disk-facts/target/release/examples/mention_candidates'
PINS={TRAIN:'25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b',GRAPH:'fc04ca4ac6f30468f619395a310181e23ebb45a44f776b81bb74178b357f0172'}


def identity(p):
    with p.open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
    return dict(path=str(p),bytes=p.stat().st_size,sha256=digest)


def dump(name,value):
    with (RUN/name).open('x') as f:json.dump(value,f,indent=2,ensure_ascii=False);f.write('\n')


def main():
    started=time.monotonic();assert not (RUN/'extractor').exists()
    proof=json.loads((RUN/'pre-run.json').read_text())
    assert identity(Path(__file__))['sha256']==proof['scripts']['prepare.py']
    identities=[]
    for p,expected in PINS.items():
        record=identity(p);assert record['sha256']==expected,p;identities.append(record)
    assert identity(BINARY)['sha256']==proof['binary_sha256']
    shutil.copyfile(BINARY,RUN/'extractor');(RUN/'extractor').chmod(0o555)
    dump('inputs.json',identities+[identity(RUN/'extractor')])
    # Pull only question columns for normalization and exclusion.
    lines=TRAIN.read_text().splitlines();assert len(lines)==19481
    questions=[line.rsplit('\t',1)[1] for line in lines]
    normalized_run=subprocess.run([str(RUN/'extractor'),'--normalize'],input=''.join(q+'\n' for q in questions),text=True,capture_output=True,check=True,timeout=30)
    normalized=[json.loads(line) for line in normalized_run.stdout.splitlines()]
    assert len(normalized)==len(questions)
    calibration=set(range(0,len(questions),10));assert len(calibration)==1949
    excluded_groups={normalized[i] for i in calibration}
    overlap=[i for i in range(len(questions)) if i not in calibration and normalized[i] in excluded_groups]
    selected=[i for i in range(len(questions)) if i not in calibration and normalized[i] not in excluded_groups]
    dump('selection.json',dict(calibration_indices=sorted(calibration),excluded_normalized_overlap_indices=overlap,fit_indices=selected,fit_question_groups=len({normalized[i] for i in selected})))
    question_path=RUN/'fit-questions.txt';question_path.write_text(''.join(questions[i]+'\n' for i in selected))
    command=[str(RUN/'extractor'),str(GRAPH)]
    with question_path.open('rb') as source,(RUN/'candidates.jsonl').open('xb') as out,(RUN/'extract.err').open('xb') as errors:
        process=subprocess.run(command,stdin=source,stdout=out,stderr=errors,timeout=1500)
    # Hash before any annotation join or diagnostic sample selection.
    dump('extraction.json',dict(command=command,returncode=process.returncode,questions=identity(question_path),candidates=identity(RUN/'candidates.jsonl'),seconds=time.monotonic()-started))
    assert process.returncode==0
    stats=Counter();groups={};examples={};max_spans=0;seen=0
    with (RUN/'candidates.jsonl').open() as f:
        for index,line in zip(selected,f,strict=True):
            record=json.loads(line);assert record['question']==questions[index] and record['normalized']==normalized[index] and record['parity'] is True
            fields=lines[index].split('\t',3);assert len(fields)==4
            gold=fields[0];baseline=record['baseline_entity'];spans=record['candidates']
            eligible={s['eligible_entity'] for s in spans if s['eligible_entity'] is not None}
            gold_spans=[s for s in spans if s['eligible_entity']==gold]
            status='right' if baseline==gold else 'unlinked' if baseline is None else 'wrong'
            recoverable=bool(gold_spans) and baseline!=gold
            stats['rows']+=1;stats['baseline_'+status]+=1;stats['has_correct_eligible_span']+=bool(gold_spans)
            stats['recoverable_'+status]+=recoverable;stats['matched_span_occurrences']+=len(spans)
            stats['ambiguous_span_occurrences']+=sum(s['ambiguous'] for s in spans)
            stats['unique_eligible_entities']+=len(eligible)
            max_spans=max(max_spans,len(spans));seen+=1
            group=groups.setdefault(record['normalized'],dict(rows=0,has_correct_span=False,recoverable=False,gold_entities=set()))
            group['rows']+=1;group['has_correct_span']|=bool(gold_spans);group['recoverable']|=recoverable;group['gold_entities'].add(gold)
            category=status+('_recoverable' if recoverable else '_covered' if gold_spans else '_missing')
            if len(examples.setdefault(category,[]))<12:
                examples[category].append(dict(train_index=index,question=record['question'],gold=gold,baseline=baseline,candidates=spans))
    assert seen==len(selected)
    correct_groups=sum(g['has_correct_span'] for g in groups.values());recoverable_groups=sum(g['recoverable'] for g in groups.values())
    report=dict(counts=dict(stats),fit_rows=len(selected),fit_normalized_groups=len(groups),calibration_rows_excluded=len(calibration),overlap_rows_excluded=len(overlap),groups_with_correct_eligible_span=correct_groups,groups_with_recoverable_baseline_miss=recoverable_groups,groups_with_conflicting_gold_entities=sum(len(g['gold_entities'])>1 for g in groups.values()),maximum_matched_spans=max_spans,all_native_link_parity=True,coverage_prerequisite_passed=correct_groups>=500 and recoverable_groups>=100,seconds=time.monotonic()-started,limitation='Oracle TRAIN-FIT candidate coverage only; no learned accuracy, CAL prediction or default change.')
    dump('fit-examples.json',examples);dump('coverage-report.json',report)
    print(json.dumps(report),flush=True)


if __name__=='__main__':main()
