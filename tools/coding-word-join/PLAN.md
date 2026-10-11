# Signature-directed word joining

Frozen before production edits,2026-10-11. Independent work while full public
facts acquisition runs; no source/database changes on that track.

Approved RustBench TRAIN(f79e71a, MIT OR Apache-2.0) contains a one-argument
&str -> String stub requiring reversed whitespace-separated words. The current
signature generator tries chars/words, original/reversed order, but every String
collector concatenates elements. Existing hurried/terse TRAIN requests reach the
native verifier and exhaust zero passing candidates. Chatty request is outside
the current grammar and remains outside scope.

Hypothesis: adding a typed word-sequence join operator with a single-space
separator, in either order, supplies a general missing composition. Generate
only from the structural signature. Never read function name, English, comments,
tests, expected outputs or task identity to choose a body. Keep every existing
candidate, the32candidate cap,90second proposal budget, opaque native tests,
exactly-one-survivor acceptance, target execution check, live recheck and rollback.
Do not expand request grammar or alter any test/verifier.

Freeze authored native fixtures before editing: normalization of whitespace in
original word order; reversed word order; Unicode/punctuation/empty/one-word,
repeated words and multiline whitespace; old character reversal and concatenation
with distinct behaviors despite identical signatures; ambiguous examples where
new and old candidates both pass must refuse unchanged; already implemented,
no target-executing tests and unsupported signatures must refuse unchanged.
Preserve fixture source, manifest and tests except the selected stub on success.

Acceptance: baseline refuses the two new authored behaviors and current TRAIN
reverse-words styles; candidate correctly implements both authored behaviors and
all existing accepted native contracts. Ambiguity/unsupported cases remain
unchanged. Fixed TRAIN sample includes all3reverse-words and all3sum-even styles;
then full60 with >=2new verified implementations, all3prior accepted outputs
preserved byte-for-byte,0failed verification. Check all affected world tests,
requiredfmt/clippy/release/strict4smoke. Only after these pass run unchanged
full765codingDEV: retain all218passes, gain>=2, zero failed verification. Report
TRAIN/task overlap honestly; not an unseen-task generalization claim. No held-out
or expected patches, no new downloads/models/workers/APIcalls. Each runner finite,
every cargo8GiB/no swap/setsid--wait/on-diskTMPDIR, atmost4testthreads and1buildjob.
