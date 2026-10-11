# Frozen plan (2026-10-11, before runtime edits)
Hypothesis: replacing a syntactically resolved unsigned primitive quotient with
::core::primitive::TYPE::div_ceil(left, right), once per operand in original order,
repairs partial-unit rounding. Append after ALL legacy proposals, including */.
Resolve conservative typed parameters and same-file direct struct fields; infer
unsuffixed integer literals only from the other proven unsigned operand. Reject
custom/signed/unknown/shadowed contexts, macros and protected test code. Extend
only exact introduced calls in the existing checked scalar ambiguity interpreter;
keep both native witness replays, all budgets and rollback unchanged.
Acceptance: >=8 exact authored native positives; zero negative/rollback leaks;
native-confirmed two-quotient ambiguity withdrawal; public fixed sample then full
27: >=1 newly verified, no loss or changed old patches without justified ambiguity.
Parent opaque full765 (not accessible here) must retain 200 passes/0 failures;
parent integration remains conditional on that gate. Freeze original submission
source/binary/patch/results before any rejection rollback. No new downloaded data.

Fixtures are frozen in fixtures.json by generate.py before runtime edits. Labels
and exact expected edits immutable. Six primitive widths each exercise 0, exact,
inexact, maximum and guarded zero divisor. Additional typed fields and Unicode
cases bring positives to eight. Negatives cover signed, unknown local binding,
custom Div plus div_ceil, macro and test protection, impossible visible tests
(native failed verification rollback). Underconstrained two-quotient case requires
withdrawal; legacy multi-file + and */ proposals retain global ordering.
