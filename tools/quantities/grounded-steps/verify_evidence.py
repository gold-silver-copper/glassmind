#!/usr/bin/env python3
"""Recount frozen TRAIN audit and check evidence links; not independent contracts."""
import argparse
from collections import Counter, defaultdict
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import re


def read(path):
    with path.open() as f:
        return [json.loads(x) for x in f]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source', type=Path, required=True)
    p.add_argument('--frozen', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    source = read(a.source/'train.jsonl')
    full = a.frozen/'full'
    records = read(full/'records.jsonl')
    members = read(full/'membership.jsonl')
    requests = read(full/'question-only-binding.requests.jsonl')
    bindings = read(full/'question-only-binding.outputs.jsonl')
    goldrequests = read(full/'gold-provided-execution.requests.jsonl')
    goldoutputs = read(full/'gold-provided-execution.outputs.jsonl')
    goldids = read(full/'gold-provided-execution.row-ids.jsonl')
    assert len(source)==len(records)==len(members)==len(requests)==len(bindings)==7473
    constants = list(map(Fraction,['0','1','2','3','4','5','6','10','12','26','52','60','100','180','360','1000','3600','0.25','3.6','1.6']))
    groups = defaultdict(set)
    collisions = Counter()
    eligible_collisions = Counter()
    leaf_counts = Counter()
    for i, (s,r,m,req,b) in enumerate(zip(source,records,members,requests,bindings)):
        assert r['row_id']==m['row_id']==i
        assert req=={'question':s['question'],'program':None}
        norm = re.sub('[0-9]+','#',' '.join(s['question'].lower().split()))
        group = hashlib.sha256(norm.encode()).hexdigest()
        assert m['normalized_question']==norm and m['group_sha256']==r['group_sha256']==group
        assert m['subset']==r['subset']==('TRAIN-CAL' if int(group,16)%5==0 else 'TRAIN-FIT')
        groups[group].add(m['subset'])
        if b['status']=='bound':
            for n in b['numbers']:
                assert s['question'].encode()[n['start']:n['end']].decode()==n['literal']
                assert Fraction(n['literal'].replace(',',''))==Fraction(n['value'])
        if 'leaves' not in r:
            continue
        product = 1
        row_collisions = set()
        for leaf in r['leaves']:
            value = Fraction(leaf['value'])
            choices = []
            pools = [('input',[Fraction(n['value']) for n in b['numbers']]),
                     ('step',[Fraction(n['value']) for n in r['expression_nodes'][:leaf['available_before']]]),
                     ('constant', constants)]
            for kind, values in pools:
                choices.extend({'kind':kind,'index':j} for j,v in enumerate(values) if v==value)
            assert choices==leaf['alternatives']
            product *= len(choices)
            leaf_counts[str(len(choices))] += 1
            if len(choices)>1:
                kinds = Counter(x['kind'] for x in choices)
                for k, count in kinds.items():
                    if count>1: row_collisions.add(k+'/'+k)
                for k in kinds:
                    for other in kinds:
                        if k<other: row_collisions.add(k+'/'+other)
        assert str(product)==r['candidate_program_count']
        collisions.update(row_collisions)
        if not (set(r['reasons'])-{'ambiguous_leaf'}):
            eligible_collisions.update(row_collisions)
        if r['accepted']:
            assert product==1 and not r['reasons']
            assert 1<=len(r['program']['steps'])<=6
    assert all(len(s)==1 for s in groups.values())
    accepted = [r['row_id'] for r in records if r['accepted']]
    assert accepted==[x['row_id'] for x in goldids]
    assert len(accepted)==len(goldrequests)==len(goldoutputs)
    for rowid, req, out in zip(accepted,goldrequests,goldoutputs):
        r = records[rowid]
        assert req=={'question':source[rowid]['question'],'program':r['program']}
        assert out['status']=='proposed' and out['semantics']=='unverified'
        assert Fraction(out['value'])==Fraction(r['final_answer'])
        assert out['numbers']==bindings[rowid]['numbers'] and out['program']==r['program']
    sample = read(a.frozen/'sample/records.jsonl')
    assert sample==records[:200]
    assert read(a.frozen/'sample/question-only-binding.outputs.jsonl')==bindings[:200]
    reason_sets = Counter(tuple(sorted(r['reasons'])) for r in records)
    report = {
        'rows':len(records),'groups':len(groups),'groups_crossing_split':0,
        'sample_full_exact_records':len(sample), 'accepted_native_exact':len(accepted),
        'question_only_requests_exact':len(requests),
        'all_recorded_candidate_sets_reconstructed':True,
        'ambiguity_row_causes_overlapping':dict(sorted(collisions.items())),
        'otherwise_eligible_ambiguity_causes_overlapping':dict(sorted(eligible_collisions.items())),
        'leaf_candidate_cardinalities':dict(sorted(leaf_counts.items())),
        'exclusive_reason_combinations':[
            {'reasons':list(k),'rows':v} for k,v in sorted(reason_sets.items(),key=lambda x:(-x[1],x[0]))],
        'by_subset_and_step_count':{}, 'by_subset_and_input_count':{},
        'by_subset_and_operator_presence':{},
        'limitations':'Internal evidence integrity check; independent coordinator contracts unavailable.'}
    for split in ('TRAIN-FIT','TRAIN-CAL'):
        rs=[r for r in records if r['subset']==split]
        for field in ('step_count','input_count'):
            groups2 = {}
            for key in sorted(set(str(r[field]) for r in rs)):
                group2=[r for r in rs if str(r[field])==key]
                groups2[key]={'rows':len(group2),'accepted':sum(r['accepted'] for r in group2),
                              'reasons':dict(Counter(x for r in group2 for x in r['reasons']))}
            report['by_subset_and_'+field][split]=groups2
        report['by_subset_and_operator_presence'][split]={op:{'rows':sum(op in r['operators'] for r in rs),
                 'accepted':sum(r['accepted'] and op in r['operators'] for r in rs)} for op in ('add','subtract','multiply','divide')}
    a.output.write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:v for k,v in report.items() if not k.startswith('by_') and k!='exclusive_reason_combinations'},sort_keys=True))


if __name__=='__main__':
    main()
