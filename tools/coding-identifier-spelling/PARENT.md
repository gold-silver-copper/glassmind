# Preserve exact names in call notation

Renaming a function to `renamed_()` previously used `renamed`, because prose typo
normalization removed underscores before the tokenizer recognized explicit code.
The tokenizer now preserves the full name when adjacent `()` identifies code.
Ordinary prose normalization is unchanged.

The frozen eight-request audit improves from five safe cases and three wrong
names to eight safe cases. All five previously correct edits/refusals remain
byte-identical; all three wrong names are corrected by actual edits. Six native
integration tests execute and pass, with exact source and unrelated-file checks.
All 41 scoped English tests, workspace fmt/clippy/release and strict4/4 smoke
pass. Combined coding commit5f2859a retains all174passes on full765, with591refusals
and0failures. These authored checks are not a benchmark score.

The legacy reading/grammar parse input audit finds no underscore-call notation
in any of 2,680 grammar sentences or 120 reading documents. Those inputs therefore
cannot take the changed tokenizer branch. The new grammar parse baseline is
retained for reviewing the separate identifier-role proposal; byte probability
and passage-QA tokenizers are separate and unchanged.

Original runtime candidate, raw fixtures and native logs:
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/identifier-spelling`.
Baseline fixtures: sibling `identifierroles-review/baseline`. Small reports here
include exact identities. New local probes use the workspace license; no data
was downloaded and no held-out material was accessed.

Reproduce the exact spelling probe with `probes.py BINARY NEW_OUTPUT_DIRECTORY`
under a memory8GiB/zero-swap scope, persistent TMPDIR and external target,
setsid --wait, timeout --kill-after=10s600. Then `behavior.py` replays native tests
on copies of those retained candidate fixtures. The native replay sets a distinct
target for each copy to avoid cross-fixture cache reuse. Scoped test counts are
summarized from the tool transcript; that JSON is explicitly not a raw test log.
