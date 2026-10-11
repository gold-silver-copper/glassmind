# Preregistered bounded batch

Source eb22519f20325e72549a103cf99180ea0dbc2ba3; corrected repairambiguity2-run only.
Inspected the 27 original TRAIN requests, supplied production Rust, original
visible integration tests for the scalar example, existing slips/checks and
prior coding-repair report. No hidden assets or old private logs read.

Hypothesis: two test-passing mutations with a concrete differing defined scalar
execution establish underconstraint, warranting withdrawal of the first repair.
No expected output is inferred. No domain/name-specific rule.

Representation: one top-level, non-test, non-attributed (doc comments allowed),
non-generic safe free function, one immutable scalar argument, scalar return,
a single tail expression. Fixed-width signed/unsigned integers up to 64 bits,
bool; parentheses, literals, argument references, arithmetic + - * / %,
comparisons, &&/||/!, unary minus. Fully validate both expression trees and types
before evaluation. No calls, macros, locals, casts, floats, loops or side effects.
Check Rust numeric bounds at every arithmetic step; undefined/panicking runs
are excluded, and boolean operations short circuit. Unsupported = no evidence.

Bounds frozen before editing: first passing candidate unchanged; inspect up to
8 subsequent same-file candidate trials with supported differing behavior.
At most 512 deterministic typed inputs: small range -128..255, type boundaries,
and source literal neighbors, truncated deterministically. Entire extra search
has a 10-second admission budget, shares existing 120-second search budget,
and must not start past elapsed 80 seconds (parent turn budget 100 seconds).
Cargo's existing individual 600s timeout remains a known coarse budget limit.
Do not increase the existing timeout or candidate ordering. No evidence means
return the original first candidate. Sampled agreement is not equivalence.
Retain exact source, two patches, two output values, input and type in JSON;
report refusal evidence on stderr without changing CLI or response schemas.

Acceptance: at least one concretely ambiguous repair withdrawn; all other
previously accepted repairs byte-identical absent their own witnesses; no new
claims. Preserve independently justified fixtures. Fixed sample then full27
using unchanged coding-repair-dev.py. Original visible tests remain byte-identical.
Independent fixtures: ambiguous and unique repair, overflow/type bounds,
short circuit, unsupported calls, test body exclusion. Coordinator alone runs
full765 gate: preserve all172 passes and reduce3 failures, no new failures.
No external data/downloads; source f79e71a MIT OR Apache-2.0 per prior provenance;
generated TRAIN requests have no separately declared license.
