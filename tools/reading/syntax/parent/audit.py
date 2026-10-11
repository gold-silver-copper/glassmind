"""Independent scoring of retained forms-only dependency predictions."""
from pathlib import Path
import argparse, collections, hashlib, json


def corpus(path):
    result, rows = [], []
    for line in path.read_text().splitlines() + ['']:
        if not line:
            if rows:
                result.append(rows)
                rows = []
        elif not line.startswith('#'):
            fields = line.split('\t')
            assert len(fields) == 10
            if fields[0].isdigit():
                assert int(fields[0]) == len(rows) + 1
                rows.append((fields[1], fields[3], int(fields[6]), fields[7]))
    return result


def is_tree(heads, labels):
    n = len(heads)
    if n == 0 or len(labels) != n or heads.count(0) != 1:
        return False
    for i, (head, label) in enumerate(zip(heads, labels), 1):
        if type(head) is not int or head < 0 or head > n or head == i or (head == 0) != (label == 'root'):
            return False
        seen, node = {i}, head
        while node:
            if node in seen or node < 0 or node > n:
                return False
            seen.add(node)
            node = heads[node-1]
    return True


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--source',type=Path,required=True)
    p.add_argument('--predictions',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    a=p.parse_args()
    train=corpus(a.source/'en_ewt-ud-train.conllu')
    dev=corpus(a.source/'en_ewt-ud-dev.conllu')
    predictions=[json.loads(x) for x in a.predictions.read_text().splitlines()]
    assert len(dev)==len(predictions)==2001
    seen={tuple(x[0] for x in sent) for sent in train}
    groups=collections.defaultdict(collections.Counter)
    ledger=[]
    for number,(gold,pred) in enumerate(zip(dev,predictions)):
        forms=[x[0] for x in gold]
        duplicate=tuple(forms) in seen
        emitted=pred.get('status')=='predicted'
        if emitted:
            assert pred['forms']==forms and pred['tokenization']=='provided_forms'
            assert pred['spans'] is None
            assert len(pred['upos'])==len(forms)
            assert is_tree(pred['heads'],pred['deprels'])
        row=collections.Counter()
        for i,(_,pos,head,label) in enumerate(gold):
            uas=emitted and pred['heads'][i]==head
            las=uas and pred['deprels'][i]==label
            for group in ['all','duplicate_forms' if duplicate else 'nonduplicate_forms'] + ([] if pos=='PUNCT' else ['exclude_punctuation']):
                groups[group]['words']+=1
                groups[group]['uas_correct']+=uas
                groups[group]['las_correct']+=las
            row['uas_correct']+=uas
            row['las_correct']+=las
        root=next(i for i,x in enumerate(gold) if x[2]==0)
        for group in ['all','duplicate_forms' if duplicate else 'nonduplicate_forms']:
            groups[group]['sentences']+=1
            groups[group]['emitted']+=emitted
            groups[group]['root_correct']+=emitted and pred['heads'][root]==0
        ledger.append(dict(sentence=number,words=len(gold),duplicate=duplicate,emitted=emitted,**row))
    result={'predictions_sha256':hashlib.sha256(a.predictions.read_bytes()).hexdigest(),'gold_tokenized':True,'groups':{k:dict(v) for k,v in groups.items()},'independent_tree_validation':True,'unsupported_counted_as_misses':True}
    a.out.mkdir(parents=True,exist_ok=True)
    (a.out/'score.json').write_text(json.dumps(result,indent=2)+'\n')
    (a.out/'ledger.jsonl').write_text(''.join(json.dumps(x)+'\n' for x in ledger))
    print(json.dumps(result,indent=2))
    full=groups['all']
    assert full['words']==25148 and full['las_correct']/full['words']>=.45 and full['uas_correct']/full['words']>=.55


if __name__=='__main__':
    main()
