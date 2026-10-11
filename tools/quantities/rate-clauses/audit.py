"""Post-DEV arithmetic audit; no runtime imports, metric edits or new inference.
Uses integer work completed per a chosen common duration, not the Rust solver.
"""
import json
from fractions import Fraction
from pathlib import Path
p=Path(__file__).resolve().parent
changes=json.loads((p/'reports/full/new-answers.json').read_text())
# Common-duration arithmetic computed independently from each newly answered Problem.
# 2469: in36 days all complete9 jobs; A3 jobs; B2 jobs; C4 jobs =>9 days/job.
# 3869: in30 days both complete5 jobs; A3 jobs; B2 jobs =>15 days/job.
# 4126: in20 days both complete20 jobs; A1 job; B19 jobs =>20/19 days/job.
derivations={
 2469:dict(common_days=36,joint_jobs=9,known_individual_jobs=[3,2],unknown_jobs=4,expected=Fraction(9),label='9 days',unit='days'),
 3869:dict(common_days=30,joint_jobs=5,known_individual_jobs=[3],unknown_jobs=2,expected=Fraction(15),label='15',unit='days'),
 4126:dict(common_days=20,joint_jobs=20,known_individual_jobs=[1],unknown_jobs=19,expected=Fraction(20,19),label='0.95 days',unit='days')}
assert set(derivations)=={x['index'] for x in changes}
for row in changes:
 d=derivations[row['index']]
 assert d['joint_jobs']-sum(d['known_individual_jobs'])==d['unknown_jobs']
 assert Fraction(d['common_days'],d['unknown_jobs'])==d['expected']
 assert Fraction(row['candidate']['value'])==d['expected'] and row['candidate']['unit']==d['unit']
 d['expected']=str(d['expected']);d['question']=row['question'];d['candidate']=row['candidate'];d['v3_grade']=row['candidate_grade']
# Selected label19/20 days on4126 is reciprocal of the duration; it equals
# the unknown agent's rate in jobs/day, not a duration. Gate remains failed.
out=dict(arithmetic_and_units_verified=3,metric_correct=2,metric_wrong=1,
 label_discrepancy='4126: given1day joint and20days for A imply20/19days for B; selected0.95days is19/20, numerically the jobs/day rate. No scorer or label was changed.',
 independent_reviewer=False,derivations=derivations)
(p/'audit.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
