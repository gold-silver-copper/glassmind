#!/usr/bin/env python3
"""Post-evaluation evidence inventory, no scoring or acceptance changes."""
import json
from pathlib import Path
from collections import Counter
import experiment as ex


def main():
    fetch=json.loads((ex.RUN/'fetch.json').read_bytes())
    merge=json.loads((ex.RUN/'merge.json').read_bytes())
    sample=json.loads((ex.RUN/'sample/report.json').read_bytes())
    full=json.loads((ex.RUN/'full/report.json').read_bytes())
    transitions=json.loads((ex.RUN/'full/transitions.json').read_bytes())
    starts=[r['start_elapsed_seconds'] for r in fetch['requests']]
    base=json.loads((ex.BASE/'scope1000/index.json').read_bytes())['entities']
    candidate=json.loads((ex.RUN/'candidate-index.json').read_bytes())['entities']
    assert all(candidate[q]['aliases']==e['aliases'] for q,e in base.items())
    assert all(not e['aliases'] for q,e in candidate.items() if q not in base)
    assert all(candidate[q]['claims'][p][:len(v)]==v for q,e in base.items() for p,v in e['claims'].items())
    report=dict(source='b569ac0eb0bab5a8b9d39cda904c816e77c08e63',
        decision='accepted' if full['accepted'] else 'rejected; evidence only, no deployment or default replacement',
        selection=ex.identity(ex.OWN/'selection.json'),fetch=dict(requests=len(fetch['requests']),
            new_decompressed_bytes=fetch['new_decompressed_bytes'],elapsed_seconds=fetch['elapsed_seconds'],
            minimum_request_start_spacing=min(b-a for a,b in zip(starts,starts[1:])) if len(starts)>1 else None,
            stop_reason=fetch['stop_reason'],started_utc=fetch['started_utc'],finished_utc=fetch['finished_utc'],
            statuses=dict(Counter(q['status'] for q in fetch['queries']))),
        graph=dict(base_entities=len(base),candidate_entities=len(candidate),new_nodes=merge['new_nodes'],
            independent_edges=merge['independent_edges'],base_aliases_preserved=True,new_aliases_empty=True,
            base_claim_lists_preserved_as_prefix=True),sample=sample,full=full,
        all_changed=[t for t in transitions if t['before']!=t['after']],
        all_new_wrong=[t for t in transitions if any(t[k] in ('right->wrong','refused->wrong') for k in ('answer','joint'))],
        all_old_correct_lost=[t for t in transitions if any(t[k].startswith('right->') and t[k]!='right->right' for k in ('answer','joint'))],
        limitations=['Main WDQS endpoint only; all incomplete/truncated sets remain incomplete.',
            'Sampled statement witnesses; no temporal/qualifier truth or global uniqueness guarantee.',
            'Optional numeric revisions may be absent; exact raw replies and request timestamps retained.',
            'Two pre-DEV local parser corrections, no additional budget or DEV tuning.',
            'No held-out access; baseline999 is rejected experimental series, unrelated to historical500.'])
    ex.write(ex.RUN/'summary.json',report)
    inventory=[ex.identity(p) for p in sorted(ex.RUN.rglob('*')) if p.is_file() and p.name!='artifact-inventory.json']
    ex.write(ex.RUN/'artifact-inventory.json',inventory)
    ex.write(ex.OWN/'RESULTS.json',report)
    print(json.dumps(dict(decision=report['decision'],fetch=report['fetch'],graph=report['graph'],full=full),indent=2))


if __name__=='__main__':main()
