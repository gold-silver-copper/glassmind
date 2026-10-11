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
