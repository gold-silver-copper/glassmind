"""Postfreeze audit, not imported by evaluator. No scoring-rule modifications.
Independent integer-pair arithmetic for every answered label and changed mixed
scalar; checks frozen evidence, without importing metric.py or reading questions.
"""
import argparse
from decimal import Decimal
import hashlib
import json
from math import gcd
from pathlib import Path

HERE=Path(__file__).resolve().parent

def pair(n,d=1):
    assert d>0
    g=gcd(n,d)
    return n//g,d//g

def literal(text):
    # Audit trusted accepted labels only. This is NOT an alternative acceptance grammar.
    text=text.strip().replace(',','')
    sign=-1 if text.startswith('-') else 1
    text=text.lstrip('+-')
    if '/' in text:
        numerator,denominator=text.split('/')
        parts=numerator.split()
        d=int(denominator)
        n=int(parts[-1])
        if len(parts)==2:
            assert 0<n<d
            n+=int(parts[0])*d
        else: assert len(parts)==1
        return pair(sign*n,d)
    d=Decimal(text)
    digits=d.as_tuple().digits
    n=int(''.join(str(x) for x in digits))
    exponent=d.as_tuple().exponent
    return pair(sign*n*10**max(exponent,0),10**max(-exponent,0))

def mul(a,b): return pair(a[0]*b[0],a[1]*b[1])
def show(a): return str(a[0]) if a[1]==1 else f'{a[0]}/{a[1]}'

def main():
    p=argparse.ArgumentParser(); p.add_argument('--reports',type=Path,default=HERE/'reports/full')
    p.add_argument('--output',type=Path,default=HERE/'reports/independent-review.json'); a=p.parse_args()
    ledger=[json.loads(x) for x in (a.reports/'ledger.jsonl').read_text().splitlines()]
    changes=json.loads((a.reports/'changed-grades.json').read_text())
    assert len(ledger)==4475 and [r['index'] for r in ledger]==list(range(4475))
    # Independently derived dictionary sufficient for the actual answered labels;
    # no index-based answers or gold selection. Unsupported audit unit must fail.
    units={'sec':('time',(1,1)), 'seconds':('time',(1,1)),
           '%':('percentage_points',(1,1)), 'percent':('percentage_points',(1,1)),
           'km':('length',(10**3,1))}
    # A separate arithmetic implementation verifies all accepted normalizations.
    reference={}
    def add(dim,scale,aliases):
        for alias in aliases.split('|'): reference[alias]=(dim,scale)
    add('time',pair(1),'s|sec|secs|second|seconds')
    add('time',pair(60),'min|mins|minute|minutes')
    add('time',pair(60*60),'h|hr|hrs|hour|hours')
    add('time',pair(24*60*60),'day|days')
    add('length',pair(1,1000),'mm|millimeter|millimeters|millimetre|millimetres')
    add('length',pair(1,100),'cm|centimeter|centimeters|centimetre|centimetres')
    add('length',pair(1),'m|meter|meters|metre|metres')
    add('length',pair(1000),'km|kilometer|kilometers|kilometre|kilometres')
    add('length',pair(254,10000),'in|inch|inches')
    add('length',pair(12*254,10000),'ft|foot|feet')
    add('length',pair(5280*12*254,10000),'mi|mile|miles')
    add('mass',pair(1,1000),'g|gm|gms|gram|grams')
    add('mass',pair(1),'kg|kgs|kilogram|kilograms')
    add('speed',pair(1),'m/s|mps')
    add('speed',pair(1000,60*60),'km/h|km/hr|kmph|kph')
    add('speed',pair(5280*12*254,10000*60*60),'mph')
    add('percentage_points',pair(1),'%|percent|percentage points')
    normalized=[]
    for row in ledger:
        gold=row['gold']
        if gold['case']=='unsupported': continue
        raw=row['selected']
        if gold['case']=='explicit_unit':
            unit=gold['unit']; assert raw.lower().endswith(unit)
            raw=raw[:-len(unit)].strip()
            dimension,scale=reference[unit]
            assert dimension==gold['dimension'] and show(scale)==gold['scale']
        else: scale=pair(1)
        coefficient=literal(raw); canonical=mul(coefficient,scale)
        assert show(coefficient)==gold['coefficient'] and show(canonical)==gold['canonical']
        normalized.append(dict(index=row['index'],selected=row['selected'],literal=raw,
                               independent_coefficient=show(coefficient),independent_scale=show(scale),
                               independent_canonical=show(canonical),normalization_changed=row['normalization_changed']))
    assert len(normalized)==3631
    verified=[]; changed_answered=[]; mixed=[]
    for row in ledger:
        gold=row['gold']; selected=row['selected']
        if gold['coefficient'] is not None and row['legacy_value']!='None' and row['legacy_value']!=gold['coefficient']:
            raw=selected if gold['case']=='bare_scalar' else selected[:-len(gold['unit'])].strip()
            value=literal(raw)
            assert show(value)==gold['coefficient']
            mixed.append(dict(index=row['index'],selected=selected,legacy_value=row['legacy_value'],
                              independently_derived=show(value),method='signed whole plus proper numerator/denominator'))
        for series,result in row['series'].items():
            actual=result['actual']
            if actual['status']!='answer': continue
            assert gold['case']!='unsupported'
            if gold['case']=='bare_scalar':
                g=literal(selected); v=literal(actual['value']); reason='numeric coefficient only; prediction dimension ignored'
            else:
                # Read explicit suffix from the label itself using fixed exact aliases.
                matches=[u for u in units if selected.endswith(u)]
                assert len(matches)==1, selected
                u=matches[0]; gd,gs=units[u]; pd,ps=units[actual['unit']]
                g=mul(literal(selected[:-len(u)]),gs)
                v=mul(literal(actual['value']),ps)
                assert gd==gold['dimension'] and gd==pd
                reason=f'explicit {u} and {actual["unit"]}; same dimension {gd}; gold scale {show(gs)}, prediction scale {show(ps)}'
            assert show(g)==gold['canonical']
            grade='correct' if g[0]*v[1]==v[0]*g[1] else 'wrong'
            assert grade==result['grade'],row['index']
            evidence=dict(index=row['index'],series=series,selected=selected,actual=actual,
                          independently_derived_gold=show(g),independently_derived_prediction=show(v),
                          grade=grade,derivation=reason,normalization_changed=row['normalization_changed'])
            verified.append(evidence)
            if row['normalization_changed']: changed_answered.append(evidence)
    assert len(verified)==56 # 25 baseline plus 31 candidate records
    expected={(x['series'],x['index']) for x in changes}
    derivations=[x for x in verified if (x['series'],x['index']) in expected]
    assert len(derivations)==len(changes)==5
    for evidence in derivations:
        # Explicit seconds on both sides: no question or solver semantics asserted.
        assert evidence['grade']=='correct'
    result=dict(status='PASS',independent_arithmetic='integer pairs, Decimal decomposition, cross multiplication; no metric imports',
       accepted_normalizations_checked=len(normalized),answered_records_checked=len(verified),changed_grade_records_checked=len(derivations),
       changed_answered_normalizations_checked=len(changed_answered),changed_grade_derivations=derivations,
       all_changed_answered_normalizations=changed_answered,all_answered_derivations=verified,
       mixed_scalar_derivations=mixed,
       evidence_sha256={n:hashlib.sha256((a.reports/n).read_bytes()).hexdigest() for n in ['ledger.jsonl','changed-grades.json','gold-normalization-changes.jsonl','summary.json']},
       limitation='Independent arithmetic implementation in same worker, not an independent human/parent review. No question-semantic verification.')
    a.output.parent.mkdir(parents=True,exist_ok=True)
    a.output.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    a.output.with_name('independent-normalization-derivations.jsonl').write_text(''.join(json.dumps(x,sort_keys=True)+'\n' for x in normalized))
    print(json.dumps({k:result[k] for k in ['status','accepted_normalizations_checked','answered_records_checked','changed_grade_records_checked','changed_answered_normalizations_checked']}))

if __name__=='__main__': main()
