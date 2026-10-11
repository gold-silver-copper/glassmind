"""Post-freeze diagnostic reports; no rule changes or alternative scores."""
from collections import Counter
from fractions import Fraction
import json
from pathlib import Path
import re
import sys
from metric import parse_gold, UNITS
HERE=Path(__file__).resolve().parent
FULL=Path(sys.argv[1]) if len(sys.argv)>1 else HERE/'full'
rows=[json.loads(x) for x in (FULL/'gold-audit.jsonl').read_text().splitlines()]
new=[]; values=[]; categories=Counter()
for r in rows:
    if r['legacy_value']=='None':
        s=r['selected'] or ''
        if r['value'] is not None: category='now_parseable:'+r['reason']
        elif r['gold_unit']: category='recognized_unit_but_'+r['reason']
        elif re.search(r'\bnone\b|\bneither\b',s): category='none_or_neither'
        elif re.match(r'\s*(?:\$|rs\b|rs\.|s\s*\.)',s): category='currency_prefix'
        elif re.search(r'\d\s*:\s*\d',s): category='colon_ratio_or_time'
        elif re.search(r'\b(?:sq|square|cu|cubic)\b|[²³]|\^',s): category='powers_or_area_volume'
        elif re.search(r'\b(?:pi|sqrt)\b|[√π+*=]',s): category='symbolic_or_compound'
        else: category='remaining:'+r['reason']
        categories[category]+=1
    if r['value'] is not None and (r['legacy_value']=='None' or r['legacy_value']!=r['value']):
        if r['reason']=='explicit_unit_conversion':
            dim,scale=UNITS[r['gold_unit']]; _,requested_scale=UNITS[r['requested_unit']]
            derivation=f"{r['raw_value']} {r['gold_unit']} * ({scale} {dim} base units/{r['gold_unit']}) / ({requested_scale} {dim} base units/{r['requested_unit']}) = {r['value']} {r['requested_unit']}"
        elif r['reason']=='percentage_points':
            derivation=f"Exact numeric token {r['selected'][:-1].strip()} = {r['value']} percentage points; percent request, no division by 100"
        else: derivation=f"Complete selected scalar {r['selected']!r} denotes {r['value']} in requested-unit contract; no suffix removed"
        entry=dict(index=r['index'],selected=r['selected'],legacy_value=r['legacy_value'],v2_value=r['value'],derivation=derivation)
        (new if r['legacy_value']=='None' else values).append(entry)
assert len(new)==139 and len(values)==8 and sum(categories.values())==1229
# New authored review probes, deliberately after freezing. Failures do NOT tune v2.
probes=[
    ('background_unit_not_request','3 hours','The display reports duration in hours. How many meters did the car travel?'),
    ('background_percent_not_request','25 %','The percentage of waste is known. What fraction remains?'),
    ('punctuated_alternative','3 hours','How many hours, or days, does it take?'),
]
checks=[]
for name,text,q in probes:
    observed=parse_gold(text,q)
    checks.append(dict(name=name,gold=text,question=q,expected='unscorable',observed=observed,passed=observed['value'] is None))
report=dict(newly_parseable=new,changed_numeric_values=values,legacy_unparseable_diagnostic_categories=dict(categories),postfreeze_context_probes=checks,decision='DO NOT PROMOTE frozen v2: request-clause and ambiguity recognition fail authored post-freeze probes. Retain scores as diagnostic only; no retuning or runtime acceptance.')
(FULL/'review.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['newly_parseable','changed_numeric_values']},indent=2))
