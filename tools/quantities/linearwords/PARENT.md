# Coordinator semantic review and corrected candidate

Recorded before the corrected full development evaluation.

The worker returns -1 for “The first number is 2. The second number is 3.
What is the difference between the first number and the second number?”
English “difference between” does not specify an ordered subtraction. The same
assumption hides two possible signs of an unknown in difference constraints.
This rejects the worker's claimed zero semantic errors despite24 scored correct.
Its original source/results remain archived outside git.

Hypothesis: bounded absolute-difference branches integrated into the same affine
representation can handle ordered known operands, nested expressions, and
constraints with multiple solutions, answering only if every feasible solution
agrees on the target. Explicit “minus” remains ordered. At most4 differences
produce16 branches; depth16,8 constraints,2 unknowns and checked arithmetic remain.
Each solved branch checks original equalities and the chosen nonnegative signs.
Any underdetermined branch refuses conservatively rather than assuming it invalid.
Inconsistent or sign-invalid determined branches do not contribute answers.

Acceptance: repair the counterexample, independently verify differences and
ambiguous targets using finite enumeration whose range contains all roots,
retain the worker's50 composition checks and all3116 existing synthetic triples,
retain all19 baseline MathQA answer records, and still exceed16 correct by at
least3 with no new semantic errors. Count every gold mismatch as wrong. Full
4475 evaluation and main-talk regression remain required before publication.
Two scored-correct worker difference questions lack positivity constraints;
if both signs remain feasible, the corrected solver must refuse them even
though this reduces the benchmark score. No score definition changes.

The first training-concept unit check now uses explicit “minus”; new unit checks
cover signed operands, nested/target differences, ambiguous roots, disambiguation
by another equality and negative difference contradictions. This is a corrected
meaning, not a weakened assertion. Exact enumeration checks are in absolute.py.


## Final coordinator measurements

Source `687f2751ff256f2071330590345ef0652fb09baa`, quantity binary SHA256
`06a651b1817f022677f1c898ba8f71bb53eb2d6ed7ad5c42c707b356a869768a`; final CLI SHA256
`cbe42583c56482cec7f8dca21a368f6984e5ecb491757714318af692f9fcfad8`. Full4475 MathQA dev: **16→22 right,
4456→4450 refused,3 wrong unchanged**. Every19 baseline answer record is identical.
All six new values were independently recomputed with Fraction. Worker rows1410
and3079 now refuse because both signs remain possible; its24-correct total is
superseded. Fixedfirst100 stays1/99/0, all1229 unparseable gold rows remain in the
denominator, and the existing three scored errors stay counted wrong.

All3116 synthetic status/value/unit triples are unchanged. The50 independent
composition cases and55 finite-root/difference checks pass, as do all9 quantity
tests. Workspace fmt/all-clippy/release/strict4/4smoke pass. Main talk passes8/8
new mixed cases in allfour facts/model configurations, including preserving the
remembered name; old27/27 calculation/memory checks pass with/without model.
Two-variable and absolute-difference composition have local evidence only;
all six benchmark gains use one unknown. Coverage remains22 of4475.

[Integrated validation](integrated/validation.json),
[exhaustive new-answer audit](integrated/audit.json), and
[absolute-difference evidence](integrated/absolute.json). Raw artifacts remain at
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/linearwords-check`. No new datasets/dependencies/models downloaded, no held-out access.
