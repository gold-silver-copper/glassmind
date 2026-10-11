"""Independent exact SI oracle and conversation contracts; no worker inspection."""
from pathlib import Path
from fractions import Fraction as F
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parent.parent
HERE.mkdir(parents=True,exist_ok=True)
# Independently expressed exact conversion constants, confined to this test oracle.
units={
 'kg':('mass',F(1),F(0)), 'g':('mass',F(1,1000),F(0)), 'lb':('mass',F(45359237,100000000),F(0)),
 'm':('length',F(1),F(0)), 'ft':('length',F(381,1250),F(0)),
 'h':('duration',F(3600),F(0)), 'min':('duration',F(60),F(0)), 's':('duration',F(1),F(0)),
 'l':('volume',F(1,1000),F(0)), 'us gallon':('volume',F(473176473,125000000000),F(0)),
 'm/s':('speed',F(1),F(0)), 'km/h':('speed',F(5,18),F(0)), 'mph':('speed',F(1397,3125),F(0)),
 'c':('temperature',F(1),F(27315,100)), 'f':('temperature',F(5,9),F(45967,180)), 'k':('temperature',F(1),F(0)),
}
cases=[]
def add(name,q,l,lu,r,ru,op):
    dl,sl,ol=units[lu];dr,sr,orr=units[ru];assert dl==dr
    lv=F(l)*sl+ol;rv=F(r)*sr+orr;order='less' if lv<rv else 'greater' if lv>rv else 'equal'
    truth={'greater':lv>rv,'less':lv<rv,'equal':lv==rv,'at_least':lv>=rv,'at_most':lv<=rv}.get(op)
    choice=None
    if op in ['higher','lower']:choice='equal' if lv==rv else 'left' if (lv>rv)==(op=='higher') else 'right'
    cases.append(dict(name=name,question=q,status='compared',left=dict(value=l,unit=lu),right=dict(value=r,unit=ru),dimension=dl,si_left=str(lv),si_right=str(rv),ordering=order,operation=op,truth=truth,choice=choice))
add('mass_contradicts_raw_order','Is 2 kg greater than 1500 g?','2','kg','1500','g','greater')
add('inverse_mass','Is 1500 g less than 2 kg?','1500','g','2','kg','less')
add('fraction_duration','Is 1/3 hour equal to 20 minutes?','1/3','h','20','min','equal')
add('exact_us_volume','Is 1 us gallon equal to 3.785411784 litres?','1','us gallon','3.785411784','l','equal')
add('speed','Is 60 mph less than 100 km/h?','60','mph','100','km/h','less')
add('freezing','Is 32 fahrenheit equal to 0 celsius?','32','f','0','c','equal')
add('negative_affine','Is -40 fahrenheit equal to -40 celsius?','-40','f','-40','c','equal')
add('inclusive_equal','Is 0 celsius at least 32 fahrenheit?','0','c','32','f','at_least')
add('inclusive_lower','Is 1 minute at most 61 seconds?','1','min','61','s','at_most')
add('false_mass','Is 1 pound greater than 0.5 kg?','1','lb','0.5','kg','greater')
add('length_select_right','Which is longer, 3 feet or 1 metre?','3','ft','1','m','higher')
add('length_select_left','Which is shorter, 3 feet or 1 metre?','3','ft','1','m','lower')
add('mass_select_left','Which is heavier, 2 kg or 1500 g?','2','kg','1500','g','higher')
add('mass_select_right','Which is lighter, 2 kg or 1500 g?','2','kg','1500','g','lower')
add('speed_selection_equal','Which is faster, 36 km/h or 10 m/s?','36','km/h','10','m/s','higher')
add('slower','Which is slower, 10 mph or 5 m/s?','10','mph','5','m/s','lower')
add('hotter','Which is hotter, 0 celsius or 31 fahrenheit?','0','c','31','f','higher')
add('colder','Which is colder, 0 celsius or 31 fahrenheit?','0','c','31','f','lower')
add('zero_kelvin','Is -273.15 celsius equal to 0 kelvin?','-273.15','c','0','k','equal')
add('negative_magnitudes','Is -2 kg greater than -1500 g?','-2','kg','-1500','g','greater')
add('case_space','  IS 1.5 KILOGRAMS AT MOST 1500 GRAMS ?  ','1.5','kg','1500','g','at_most')
add('fraction_speed','Is 1/2 m/s equal to 1.8 km/h?','1/2','m/s','1.8','km/h','equal')
add('false_equality','Is 1 metre equal to 3 feet?','1','m','3','ft','equal')
add('false_inclusive','Is 2 minutes at most 119 seconds?','2','min','119','s','at_most')
negative={
 'cross_dimension':'Is 1 kg greater than 1 metre?',
 'ambiguous_gallon':'Is 1 gallon equal to 4 litres?',
 'unknown_unit':'Is 2 bananas greater than 1 apple?',
 'unitless':'Is 2 greater than 1?',
 'missing_right_unit':'Is 2 kg greater than 1?',
 'arithmetic_expression':'Is 1 kg plus 2 kg greater than 2 kg?',
 'wrong_selection_dimension':'Which is hotter, 2 kg or 3 kg?',
 'cross_selection':'Which is heavier, 1 litre or 1 kg?',
 'zero_denominator':'Is 1/0 kg equal to 1 kg?',
 'overflow_literal':'Is 340282366920938463463374607431768211456 kg greater than 1 kg?',
 'nonfinite':'Is NaN kg equal to 1 kg?',
 'appended_memory':'Is 2 kg greater than 1 kg? My name is Changed',
 'appended_story':'Which is longer, 2 m or 1 m? Mary moved to the office.',
 'extra_relation':'Is 2 kg greater than 1 kg less than 3 kg?',
 'missing_quantity':'Is kg greater than 1 kg?',
 'nul':'Is 2 kg greater than 1 kg?\0',
 'oversized':' '*4096+'Is 2 kg greater than 1 kg?',
}
cases += [dict(name=n,question=q,status='refused') for n,q in negative.items()]
(HERE/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
plan='''# Frozen exact unit comparison parent review

Twenty-four successful exact comparisons cover all six existing dimensions, every
requested predicate/selection form, false/equal/inclusive results, negative/fraction
magnitudes and affine temperature offsets. Seventeen unsupported/malformed inputs
must refuse. The oracle uses independent Fraction arithmetic and checks the result
and exact converted witness, never raw numeral ordering or approximate floats.
All library/direct-query/CLI semantics must agree. These are authored capability
contracts, not a new independently sourced word-problem accuracy benchmark.

Main route must expose mathematical comparisons with a typed quantity_comparison
reply and clear false/equal/left/right rendering, preserve personal and story state,
increment normal turn bookkeeping and clear correction focus. Natural supported
forms work directly; an explicit Compare quantities: prefix also reserves invalid
payloads from ordinary interpreters. No pronoun or personal-memory binding. A
malformed appended name/story assertion never updates memory or story state. Main
reply structure can adapt to the reviewed typed API; expectations cannot change.
Test both intent modes, persisted-session restarts and ordinary memory/story/calc/
facts/dialogue coexistence. Existing scalar API remains byte-identical on fixed
sample200/full4475 and synthetic3116. No new MathQA accuracy claim or label use.
'''
(HERE/'PLAN.md').write_text(plan)
binary=ROOT/'coordinator/combined-deduporder-draftsession/glassmind'
baseline=subprocess.run([str(binary),'talk','--json'],input='\n'.join(c['question'] for c in cases if c['status']=='compared')+'\n',text=True,capture_output=True,check=True,timeout=30)
rows=[json.loads(s) for s in baseline.stdout.splitlines()];assert len(rows)==24 and all(r['kind']=='refusal' for r in rows)
(HERE/'main-baseline.jsonl').write_text(baseline.stdout)
(HERE/'freeze.json').write_text(json.dumps({n:hashlib.sha256((HERE/n).read_bytes()).hexdigest() for n in ['cases.json','PLAN.md','main-baseline.jsonl','freeze.py']},indent=2)+'\n')
print(json.dumps(dict(cases=len(cases),compared=24,refused=17,baseline_refused=24,cases_sha256=hashlib.sha256((HERE/'cases.json').read_bytes()).hexdigest())))
