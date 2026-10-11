# Identifier spelling in explicit call notation

Parent discovery on validated runtime1331ac7, binary8cde5dc4: the tokenizer removes
trailing underscores from a known English word before recognizing a following
empty call. Thus `rename function caller to renamed_()` edits the code to
`renamed`, losing a significant part of the requested Rust identifier. Quoted
`renamed_()` works. This is inherited, independently reproduced before editing;
it is separate from the identifier-role worker's proposed grammar additions.

Hypothesis: once adjacent `()` establishes explicit code notation, retain the
full identifier spelling before normalization. Keep the existing trailing-
underscore typo handling for ordinary prose without code notation. No new
language forms, world changes, lookup rules or test weakening.

Fixed authored probe: eight requests from coordinator/identifierroles-review/
exact-identifiers.py, SHA256
7584bf4a7624bdd748d76f6ed4ec02fceeb094f05f2941047e2d4664acb6cc72.
Baseline: five safe, three wrong target spellings; six total edits, two refusals.
Acceptance: all eight safe, preserve every previously correct edit/refusal, and
correct all three wrong spellings with verified edits. Source bytes must match
only the exact name replacement; test files and other files unchanged. Native
external behavior assertions must execute for accepted candidate edits. This
initial eight-row probe checks exact source changes; the added independent cargo
replay checks behavior without changing the frozen source-spelling predicates.

Run scoped English tests, affected fmt/clippy/build and existing code query/edit
regressions; parent full765 before publishing, all174 passes retained and zero
failures. This is an authored product safety improvement, not a benchmark-gain
claim. Shared ordinary grammar behavior must remain unchanged. Both independent
workers remain isolated; they do not own tokens.rs. All cargo under8GiB/0swap,
setsid --wait, timeout --kill-after=10s, persistent TMPDIR, -j1/tests4.
