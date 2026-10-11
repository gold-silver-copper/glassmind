# Frozen TRAIN-only audit policy (v1)

Hypothesis: official GSM8K TRAIN calculator annotations provide >=1000 uniquely
numerically grounded typed programs under the unchanged original native executor.
Data-readiness continuation additionally requires ALL independent coordinator
contracts; these are unavailable to this worker. This is not solver accuracy,
English semantic verification, a runtime acceptance gate, or independent DEV.
No fitting, model calls, retuning, network access, or changes to the native binder.

Baseline: no converted GSM8K programs existed for this batch. Historical supplied
MathQA CAL figures: free 143 -> 92; oracle 221 -> 170; left 2197 -> 2370 and right
2499 -> 2540 /4138; 3/4 gates failed. These are not comparable GSM8K metrics.
One frozen conversion policy only; no parameter search on TRAIN-CAL.

## Source and membership

Only supplied GSM8K official train.jsonl (7473 rows), LICENSE, manifest and exact
original native executor/source are accessible inputs. Verify every supplied
file hash and byte count before running and again at handoff. Preserve all rows.
Question is passed byte-for-byte unchanged to native binding and gold execution.
Membership: Unicode lowercase, Python Unicode whitespace split/join with ASCII
space, then replace each ASCII [0-9]+ run by #. SHA256 UTF-8 normalized string,
interpret full digest big-endian integer; modulo 5 == 0 means TRAIN-CAL,
otherwise TRAIN-FIT. Identical normalized groups cannot cross subsets. Neither
subset is independent DEV. Store row index (zero based), raw line SHA, normalized
question, group hash, subset, question SHA, and answer SHA.

## Exact grammar and limits

Annotations are non-overlapping literal <<expression=value>> blocks in answer
rationale before the sole #### delimiter. All << and >> must form matched blocks;
nested or stray delimiters, extra equals, and annotation delimiters after ####
are malformed. Whitespace around expressions and values is permitted. Ignore
unannotated rationale prose; never mine it for numbers or entity roles.
Final answer: entire suffix after sole ####, stripped, must be a signed decimal
or integer (optional ASCII '-' only), optionally correctly comma-grouped thousands.
Annotation values use the same scalar syntax. No units, exponent notation,
percent signs, fractions, positive signs, or Unicode numeral/sign alternatives.

Expression grammar (ASCII whitespace permitted between tokens):
  expr := term (('+' | '-') term)*
  term := atom (('*' | '/') atom)*
  atom := unsigned_decimal | '(' expr ')'
  unsigned_decimal := [0-9]+ ('.' [0-9]+)? | '.' [0-9]+
No unary signs, variables, equals inside expression, commas, implicit product,
exponents, floor division, or eval/exec/symbolic solving. Standard precedence;
left associativity; no algebraic rewriting, simplification, or commutation.
Each annotation must contain an operator. At most 64 annotations, 1024 characters
per expression, 128 tokens per expression, nesting depth 32, 128 total operator
nodes per row (including rows too long for native programs). Numbers <=100 chars.
All arithmetic exact rational, with checked i128 operations matching original
rational.rs: cross-cancel multiply/divide, gcd-aware add/subtract, including
intermediate overflow checks before reduction. No rounding or tolerance.
Reject arithmetic overflow/division zero, any annotation value unequal to its
expression, or final value unequal to the last expression result. Wrong labels
are never corrected, omitted, or replaced. Preserve reasons and labels.

## Grounding and structural uniqueness

Expand ALL annotation trees in source order, deterministic left-child-first
postorder; emit each operator exactly once. Do not drop redundant annotations.
A syntactic non-leaf child is linked explicitly to its emitted node. For every
numeric leaf retain the FULL list of equal-valued native question occurrences,
previously completed emitted expression nodes (including completed left sibling
subtrees), and exact native constants. Future/active nodes never qualify. Empty
sets are unbound; diagnose equal-valued later nodes as impossible forward refs
and equal-valued active ancestor nodes as cycles. Such evidence never repairs a
leaf. No source precedence, no deduplication of equal-valued occurrences, no
selection of the first choice. Even input/constant, input/prior-step, and
prior-step/prior-step collisions are ambiguous. An ambiguous prefix remains
ambiguous even if later numeric values are identical. Full Cartesian program
alternatives are represented compactly by independent leaf candidate sets;
all candidates have equal rational value. Store the product of set cardinalities.

Native constants in exact order:
0,1,2,3,4,5,6,10,12,26,52,60,100,180,360,1000,3600,0.25,3.6,1.6.
Only 4 native operations add/subtract/multiply/divide, 16 input slots, 1..6 steps.
Question-only native refusal always excludes a row; never change its binding.
Check every native number's UTF-8 byte offsets, literal and value. A unique
program requires every leaf to have exactly one choice and every other gate to
pass. Send each structurally unique eligible program to the EXACT supplied
executor, separately marked gold-provided; final rational and full number records
must match. Executor refusals/mismatches exclude acceptance and remain evidence.
No answer/rationale text enters question binding or runtime features. Programs
are TRAIN-label-derived; execution requests are NOT predictions/free inference.

## Evidence and acceptance

Freeze authored contracts before corpus conversion: straightforward two-step;
repeated input value; input/prior collision; constant collision; Unicode numeral;
number word; malformed annotation; wrong intermediate; wrong final; missing
number; impossible forward reference; nested expression and overlong horizon;
division zero; UTF-8 spans. Add overflow, sign, and cycle checks as useful.
Report independent stage indicators, all exclusion reasons (overlapping, with
not-evaluated downstream stages explicit), by FIT/CAL, operator-node count,
operator presence, and native input count. Preserve every row and full binding
outputs, separate gold execution requests/outputs, source/code/command/output
hashes and membership externally at RUN/frozen-candidate. Repository contains
only this policy, authored contracts, converter, and compact report.

If unique accepted <1000, reject this supervision pipeline for continuation;
report coverage causes and a materially different next direction. Never weaken
policy or gate. If >=1000, independent contracts remain pending with coordinator.
