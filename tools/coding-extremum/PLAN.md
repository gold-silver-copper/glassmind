# Frozen plan (before production edits)
Hypothesis: a literal seed of a proven primitive signed-integer max/min reduction
can incorrectly dominate every element. Replacing only that seed with the exact
primitive MIN/MAX fixes this structural family while preserving empty => None.
Acceptance: TRAIN 7/27 -> at least 8/27 verified, all seven prior response/patch
records identical; fixed authored native contracts pass; world-rust tests, fmt,
clippy and release build pass. Passing public tests is not semantic proof.

Scope: root src/lib.rs or src/main.rs free functions, no generics, attributes
except docs, one immutable shared-reference argument to [signed integer], Vec of
signed integer, or directly declared non-generic named struct with such Vec field.
Exactly four statements: matching collection.is_empty() => return None; mutable
scalar local literal initializer (optional exact type); for &element in slice or
&Vec/field; Some(accumulator). Loop contains exactly if pure element/accumulator
strict comparison { accumulator = element; }. Both operand orders supported.
Both extrema, all six signed widths, any noncolliding names including Unicode.
No aliases/imports/modules/macros/custom attributes anywhere, or custom types
standing in for primitives; unrelated plain functions/structs permitted. Reject
all other body shapes, additional parameters, shadowing, effects, altered result,
multiple folds within a function, unknown control flow, collection mutation.
Fully qualified ::core::primitive::<type>::MIN/MAX replacement. Root bindings
named core/Option/Some/None/Vec or primitive types reject the file.

Append proposals after ALL old families. Preserve old undo_slip implementation
as a separate legacy path, with the exact old candidate set, budget and scalar
native witnesses. Only after legacy returns None try extrema with separate
bounded native checking; legacy ambiguity must not be overridden (explicit result
state if necessary). At most eight new candidates, 20-second total budget; any
incomplete verification or two passing new candidates withdraw with evidence.
Loops are never sent to scalar interpreter. No runtime semantic proof claim.

Contracts are frozen in contracts.json before implementation. All native source
fixtures compile and run failing tests initially. Positive tests exercise empty,
MIN, MAX, zero, negative/positive and mixed vectors. Two independent reductions
with an OR test produce two passing alternatives and separate native outputs.
Rejected attempts and limitations remain in RUN/frozen-candidate. No downloads,
new data, credentials, heldout assets, agents, model calls, or commits.
