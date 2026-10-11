"""Independent full TRAIN accounting, formal labels and native CAL replay."""
from pathlib import Path
from collections import Counter
import hashlib,json,struct,subprocess,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parent.parent
F=ROOT/'equivbinding-run/frozen-candidate';E=F/'experiment';out=HERE/'candidate';out.mkdir(exist_ok=False)
read=lambda p:[json.loads(s) for s in p.read_text().splitlines()]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=json.loads((F/'manifest.json').read_text())
for name,r in manifest.items():assert (F/name).stat().st_size==r['bytes'] and sha(F/name)==r['sha256']
source=ROOT/'groundingaudit-source/rejected-source/crates/quantities/src'
assert (E/'helper/src/proposal.rs').read_bytes()==(source/'proposal.rs').read_bytes()+b'\ninclude!("instrument.rs");\ninclude!("checks.rs");\n'
assert (E/'helper/src/rational.rs').read_bytes()==(source/'rational.rs').read_bytes()
assert read(E/'membership.jsonl')==read(ROOT/'groundingaudit-run/frozen-candidate/audit/membership.jsonl')
gold=read(E/'clean.jsonl');assert gold==read(ROOT/'groundingaudit-run/frozen-candidate/audit/clean.jsonl')
assert read(E/'fit-input.jsonl')==read(F/'original/fit-input.jsonl')
targets={r['index']:r for r in read(HERE/'corpus/targets.jsonl')}
metric=Path('/home/gsc/code/glassmind/tools/quantities/typed-label-metric/metric.py')
assert sha(metric)=='24f1d65ed2f1785b940d7c674565dd9b49c168e498f57c23cbd6c8c2c6eab6f0'
sys.path.insert(0,str(metric.parent));from metric import compare,parse_gold
op_hashes={};trained=[]
for arm,path in [('original',F/'original/model.json'),('A',E/'A-model.json'),('B',E/'B-model.json')]:
    model=json.loads(path.read_text());assert path.stat().st_size<=16*1024*1024
    op_hashes[arm]=hashlib.sha256(b''.join(struct.pack('<f',x) for x in model['weights'][0])).hexdigest()
    if arm!='original':
        train=json.loads((E/(arm+'-training.jsonl')).read_text());assert train['rows']==4590 and train['steps']==15753
        assert [r['epoch'] for r in train['epochs']]==list(range(12,18))
        assert all(r['rate']==0.08/(1+0.2*r['epoch']) for r in train['epochs']);trained.append(train)
assert len(set(op_hashes.values()))==1
assert trained[0]['right_contexts']==15753 and trained[1]['right_contexts']==22797

totals={};grades={};records={};target_checks=0
old={r['index']:r for r in read(ROOT/'groundingaudit-run/frozen-candidate/audit/diagnostics.jsonl')}
for arm in ['original','A','B']:
    totals[arm]={s:{k:Counter() for k in ['free','oracle','teacher']} for s in ['FIT','CAL']}
    rows=read(E/(arm+'-full.jsonl'));assert len(rows)==len(gold)==5772;records[arm]=rows
    for c,row in zip(gold,rows,strict=True):
        assert c['index']==row['index'] and c['split']==row['split'];total=totals[arm][c['split']]
        target=targets[c['index']];assert len(row['diagnostic']['teacher'])==len(target['steps'])
        for exp,actual in zip(target['steps'],row['diagnostic']['teacher'],strict=True):
            assert actual['valid_pairs']==exp['pairs'] and actual['gold_left']==exp['gold_left'] and actual['gold_right']==exp['gold_right'];target_checks+=1
            assert actual['depth']==exp['depth'] and actual['op']==['add','subtract','multiply','divide'].index(exp['op'])
            ls={l for l,r in exp['pairs']};rs={r for l,r in exp['pairs'] if l==exp['gold_left']}
            derived={'left_exact':actual['left']==exp['gold_left'],'right_exact':actual['right_gold_left']==exp['gold_right'],'left_equiv':actual['left'] in ls,'right_equiv':actual['right_gold_left'] in rs,'pair_equiv':[actual['left'],actual['right_pred_left']] in exp['pairs']}
            for k,v in derived.items():assert actual[k]==v;total['teacher'][k]+=v
            total['teacher']['decisions']+=1
        if arm=='original':assert row['free']==old[c['index']]['free'] and row['diagnostic']['oracle']==old[c['index']]['diagnostic']['oracle']
        for mode,pred in [('free',row['free']),('oracle',row['diagnostic']['oracle'])]:
            answer=dict(status='answer',value=pred['value'],unit=None) if pred['status']=='proposed' else dict(status='refusal')
            grade=compare(parse_gold(c['options'],c['correct'])[1],answer)['grade'];total[mode][grade]+=1;total[mode]['exact_ast']+=pred.get('program')==c['program']
            grades[arm,c['index'],mode]=grade
    for split,n in [('FIT',4590),('CAL',1182)]:
        for mode in ['free','oracle']:assert sum(totals[arm][split][mode][k] for k in ['correct','wrong','refused'])==n

# A new process receives only CAL question strings in free mode.
cal=[c for c in gold if c['split']=='CAL'];questions=''.join(json.dumps(c['question'])+'\n' for c in cal)
r=subprocess.run([str(F/'equivalent-binding-helper'),str(E/'B-model.json'),'--free'],input=questions,text=True,capture_output=True,check=True,timeout=120)
(out/'B-cal-free-native.jsonl').write_text(r.stdout)
actual=[json.loads(s) for s in r.stdout.splitlines()];expected=[x['free'] for x in records['B'] if x['split']=='CAL'];assert actual==expected
# Independently execute every proposed B CAL free and oracle program using the old executor.
requests=[];expected=[]
for c,row in zip(gold,records['B'],strict=True):
    if c['split']!='CAL':continue
    for pred in [row['free'],row['diagnostic']['oracle']]:
        if pred['status']=='proposed':requests.append(dict(question=c['question'],program=pred['program']));expected.append(pred)
r=subprocess.run([str(F/'original/equation_executor')],input=''.join(json.dumps(q)+'\n' for q in requests),text=True,capture_output=True,check=True,timeout=120)
(out/'B-cal-program-native.jsonl').write_text(r.stdout);actual=[json.loads(s) for s in r.stdout.splitlines()];assert len(actual)==len(expected)
for actual,pred in zip(actual,expected,strict=True):
    assert all(actual[k]==pred[k] for k in ['status','value','program'])

transitions={}
for arm in ['A','B']:
    transitions[arm]={}
    for split in ['FIT','CAL']:
        transitions[arm][split]={}
        for mode in ['free','oracle']:
            t=Counter(grades['original',c['index'],mode]+'->'+grades[arm,c['index'],mode] for c in gold if c['split']==split)
            transitions[arm][split][mode]=dict(t)
b=totals['B']['CAL'];a=totals['A']['CAL'];o=totals['original']['CAL']
gates={'free_min168':b['free']['correct']>=168,'oracle_min251':b['oracle']['correct']>=251,'free_above_A':b['free']['correct']>a['free']['correct'],'oracle_above_A':b['oracle']['correct']>a['oracle']['correct']}
for side in ['left','right']:
    for name,comp in [('original',o),('A',a)]:gates[side+'_equiv_gain_over_'+name]=b['teacher'][side+'_equiv']-comp['teacher'][side+'_equiv']>=125
assert gates==json.loads((E/'results.json').read_text())['gates'] and not any(gates.values())
report=dict(decision='rejected',frozen_files_verified=len(manifest),clean_membership_exact=5772,formal_pair_sets_checked=target_checks,original_free_and_oracle_records_exact=5772,unchanged_operation_hashes=op_hashes,native_free_CAL_exact=1182,native_proposed_free_and_oracle_programs=len(expected),totals=totals,gates=gates,transitions=transitions)
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(decision=report['decision'],formal_pair_sets_checked=target_checks,native_free_CAL_exact=1182,native_programs=len(expected),CAL={arm:totals[arm]['CAL'] for arm in totals},gates=gates),indent=2))
