# Signature-directed word-initial pipelines

Frozen before edits,2026-10-11. Separate root-owned worktree while the full public
claims storage job finishes. No workers/models/downloads.

Approved RustBench TRAIN atf79e71a contains a &str->String placeholder whose
terse request reaches the existing verifier but has zero passing candidates.
Its public documentation requests uppercase alphabetic word initials. This
motivates a general operator family, not reading documentation at runtime.

Hypothesis: compose the first character of each whitespace word, optional Unicode
alphabetic filtering, optional Unicode upper/lower casing, and original/reversed
word order. Add all12compositions for &str->String, retaining all6oldcandidates.
The generator sees only the signature. No name/task/English/doc/test selection,
no test extraction or expected-output inference. No request grammar change.
Preserve32candidate/90second budgets, exactly-one-survivor rule, targetexecution,
livecheck and rollback. Unique surviving tests do not prove intended semantics.

Freeze12independent fixtures covering every composition, mixed Unicodecase,
one-to-many uppercase, punctuation/numeric/emoji starts, nonLatinwords, irregular
whitespace, empty/oneword. Oldwordjoin10 and oldnative66contracts must remain
correct; new deliberately weak tests must refuse multiple surviving candidates.
All original files except selected placeholder must remain byte-identical.

Baselineb04a929a56f4fff3e1e748477856311a2098a5e95044b04bb1a5662fe3d99a78.
Acceptance:12new native implementations verified, all oldnativecontracts pass,
ambiguity/unexecuted/alreadyimplemented/unsupported refuse unchanged. FixedTRAIN
sample all3initials +all3sum-even +all3reverse-words styles; then full60, >=1new
implementation, all5prior successful sourceoutputs byte-identical. Independently
recompile accepted TRAIN outcomes and inspect verifier evidence. Allworldtests and
workspacefmt/clippy/release/strict4smoke pass before full765DEV. FullDEV must
retainall220passes, gain>=1,0failures. Report TRAIN/task overlap honestly; no
unseen-taskclaim. No heldout/hidden-test/expectedpatch inspection. Allcargowith
8GiB/no swap,setsid--wait,timeout,on-diskTMPDIR,jobs1/testthreads<=4.
