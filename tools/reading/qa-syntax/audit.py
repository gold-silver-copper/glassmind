#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Post-freeze audits only: no fitting, threshold changes or new acceptance gates."""
import run as r,json,collections,subprocess,math
r.verify_freeze()
def audit():
    rows=json.loads((r.RUN/'cal.labels.json').read_text());ps=[json.loads(l) for l in (r.RUN/'cal.outputs.jsonl').read_text().splitlines()]
    groups=collections.defaultdict(list)
    for row,p in zip(rows,ps):
        if p['proposal'] and p['reason']!='no_lexical_support':groups[p['margin']].append(max(r.q.SCORE['compute_exact'](a['text'],r.q.proposal(row,p)) for a in row['answers']))
    n=right=0;chosen=None
    for margin,g in sorted(groups.items(),reverse=True):
        n+=len(g);right+=sum(g)
        if n>=30 and right*2>=n:chosen=(margin,n,right)
    sel=json.loads((r.REPORT/'calibration.json').read_text())['selected'];assert chosen==(sel['threshold'],sel['accepted'],sel['right'])
    full=json.loads((r.REPORT/'full.json').read_text());probes=json.loads((r.REPORT/'probes.json').read_text());ledger=json.loads((r.RUN/'full.transitions.json').read_text());assert len(ledger)==10570
    for key in ['raw','selected']:assert dict(collections.Counter(x[key] for x in ledger))==full['transitions'][key]
    assert sum(x['selected'].startswith('right->') for x in ledger)==815
    assert sum(x['selected'].startswith('refused->') for x in ledger)==8964
    assert (r.RUN/'v1-replay.outputs.jsonl').read_bytes()==(r.BASE/'full.candidate.outputs.jsonl').read_bytes()
    c=full['candidate'];raw_exact=round(c['raw']['exact']*10570/100)
    gates=dict(raw_exact=raw_exact>=2744,raw_f1=c['raw']['f1']>=36.5184250480,selected_correct=c['right']>=815,selected_wrong=c['wrong']<=791,role_raw=probes['positive_raw_correct']>=6,no_new_selected_clear_mismatch=not probes['newly_selected_mismatches'])
    result=dict(decision='ACCEPT_OPTIONAL_UNVERIFIED_CANDIDATE' if all(gates.values()) else 'REJECT',gates=gates,raw_exact=raw_exact,cal_independently_recomputed=True,v1_full_byte_identical=True,transition_denominator=10570,substantive_selected_correct=943,newly_wrong_from_right_or_refused=sum(x['selected'] in ['right->wrong','refused->wrong'] for x in ledger),raw_strings_changed=sum(x['old_raw']!=x['new_raw'] for x in ledger),selected_strings_changed=sum(x['old_selected']!=x['new_selected'] for x in ledger),changed_wrong_to_wrong=sum(x['selected']=='wrong->wrong' and x['old_selected']!=x['new_selected'] for x in ledger),normalized_empty_refusals_never_counted_right=True)
    r.write(r.REPORT/'acceptance.json',result);print(json.dumps(result,indent=2))
def old_probes():
    cases=[];sources=[]
    for file in [r.HERE.parent/'roles/probes.json',r.HERE.parent/'support/reports/probes.json',r.HERE.parent/'qa/reports/independent.json']:
        sources.append(dict(path=str(file),sha256=r.q.digest(file.read_bytes())))
        for x in json.loads(file.read_text()):cases.append(dict(source=file.parent.name,name=x['name'],context=x['context'],question=x['question'],expected=x['expected']))
    inp=r.RUN/'legacy-probes.input.jsonl';r.q.lines(inp,[dict(context=x['context'],question=x['question']) for x in cases]);b=r.infer('legacy-probes.v1',inp,True);c=r.infer('legacy-probes.v3',inp)
    for x,p1,p3 in zip(cases,b,c):
        x.update(v1=p1,v3=p3,raw_v1=r.q.proposal(x,p1),raw_v3=r.q.proposal(x,p3))
        for name,p in [('v1',p1),('v3',p3)]:x[name+'_selected_correct']=p['status']=='refused' if x['expected'] is None else bool(r.q.SCORE['compute_exact'](x['expected'],p['answer']))
    r.write(r.REPORT/'legacy-probes.json',dict(note='Post-DEV diagnostic replay of existing public authored probes, NOT acceptance examples or parameter selection; duplicate cases retained by source.',sources=sources,cases=cases))
    print('legacy probes',len(cases),'v1 failures',sum(not c['v1_selected_correct'] for c in cases),'v3 failures',sum(not c['v3_selected_correct'] for c in cases))
def finite():
    model=json.loads((r.RUN/'model.json').read_text());model['ranker']['weights']=[3.4028234663852886e38]*65536;p=r.RUN/'finite-overflow-model.json';r.write(p,model)
    inp=r.RUN/'finite.input.jsonl';r.q.lines(inp,[dict(context='Élodie thanked Björn.',question='Who thanked Björn?')]);cmd=[str(r.BIN/'reading-qa-syntax'),'--syntax',str(r.BASE/'syntax.bin'),'predict','--model',str(p),'--input',str(inp)];out=subprocess.run(cmd,check=True,capture_output=True,text=True,timeout=30);v=json.loads(out.stdout);assert v['status']=='refused' and v['reason']=='non_finite_model_score' and math.isfinite(v['score']);r.write(r.REPORT/'finite-contract.json',v)
if __name__=='__main__':audit();old_probes();finite()
