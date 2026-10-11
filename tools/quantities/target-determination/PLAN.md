# Exact target determination under incomplete scalar constraints

Before source edits: baseline cbccc78 / quantities unchanged from PR43.
Hypothesis: exact elimination should determine the requested affine expression,
even when its operands remain free. Current solver requires full variable rank.
Keep tokenizer, consumed English grammar, two-variable/eight-equality/four-
absolute-difference limits, checked rationals and agreement across all branches.

For each branch, row-reduce equalities with skipped free columns. Detect
contradictory constant rows. Preserve existing full-rank execution. In a deficient
branch, reduce the target and sign constraints by the pivot rows, introduce the
requested target as z, and project inequalities onto z by exact Fourier-Motzkin
elimination of both remaining variables. With at most four sign inequalities
plus two target-equality halves, this is bounded (6 to <=9 to <=20 rows), with
a defensive64-row cap and checked overflow. Empty feasible set contributes no
branch; a singleton projected interval supplies an answer; any non-singleton
feasible interval refuses. This covers inequality-implied equalities as well as
ordinary row-space targets, without assuming variables are positive or integral.

Prospective gate: all authored exact/ambiguous/inconsistent cases pass, previous
50 composition and55 absolute-root contracts pass, all3116 existing synthetic
status/value/unit triples preserved, all32 existing MathQA answer payloads
preserved, zero newly wrong4475 DEV answers. Every new DEV answer independently
recomputed from question, with TRAIN/DEV disclosure. This is a general capability
improvement even if the benchmark has no new matching language. Required fmt,
affected tests/clippy/release and main-talk checks before shipping. No parser,
metric, threshold, data split or benchmark denominator change; no model call.
