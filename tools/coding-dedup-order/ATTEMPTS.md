# Attempts

Before edits: baseline full27 reproduced 8 verified/19 refused, fixed sample
2 verified/2 refused; required input SHA matched. All source manifest hashes
matched the supplied 110files/57265bytes. PLAN/cases/source/baseline frozen under
RUN/frozen-before before production edits.

Implementation attempt 1: closed typed adjacent-pair AST proposer and minimal
native-choice helper extraction. No budget/order changes to older families.
During initial test run, review tightened only the new family: compiler findings
are incomplete native verification, so cannot count as behavioral rejection.
The original first test log is retained; final-source tests will be rerun.
No rejected threshold, name specialization, expanded data or test weakening.

Clippy attempt 1 rejected four slice-index expressions under the existing
indexing-slicing lint. Replaced them with exhaustive slice destructuring; no
proposal scope or expected-output changes. Full original clippy log retained.

Review supplement (frozen separately before fix): raw identifiers must count
as the same Rust binding. Canonicalized binding counts so `r#xs` shadowing `xs`
refuses. Four additional authored native cases cover this and owned String,
char and bool. Original25 cases and expectations remain byte-identical.

Final measured result: fixed sample2/4 unchanged; full TRAIN8/27 ->9/27,
18 refusals and zero failed verifications. All eight previous entire outcome
files are byte-identical. First and final tests passed; final54 tests plus
29 authored native cases passed. All-target clippy, fmt and release passed.
Main-CLI replay independently confirmed new-vector and old-argument-swap
ambiguity refusals after both alternatives natively passed with different outputs.
No behavioral experiment was rejected; clippy failure and raw-binding review
correction above are retained. No claims about untouched coordinator full765.
