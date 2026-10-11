# TRAIN-only candidate-span preparation before a learned linker experiment

Freeze before extracting or inspecting new FIT candidates. Root-owned worktree
WORK/mention-candidates, branch continuous/mention-candidates; external outputs
WORK/coordinator/mention-candidates. WORK is
/home/gsc/.cache/glassmind/continuous-20261010. No production behavior change,
model fitting, threshold selection, CAL inference or DEV access in this stage.

The complete independent graph improves QA coverage but fails its promotion gate:
554/1949 strict CAL correct, 29 newly wrong. Twelve newly wrong cases select the
wrong entity. The prior what/which adjacency veto was rejected for removing valid
relative-clause/inverse links; do not revive it. A learned selector has never been
fitted in that earlier study because too few correct links supported calibration.

Hypothesis for preparation: whole-token aliases other than the globally longest
match often identify the annotated entity, providing enough TRAIN-only examples
to learn mention roles from question context. Measure that upper bound before
choosing any learned architecture or selection thresholds. This is an oracle
candidate-coverage diagnostic, not achieved QA improvement.

Use the same pinned official SimpleQuestions TRAIN19481, CC BY3.0, revision
 df69267f1db4f85a71325e7e8c06e192bcaf0b0e, SHA256
25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b.
Use only the complete native graph62772387840bytes SHA256
fc04ca4ac6f30468f619395a310181e23ebb45a44f776b81bb74178b357f0172.
No acquisition, added aliases, redirect changes, relation model calls or entity
selection from QA labels. Preserve the rejected fullgraph promotion experiment.

First normalize all TRAIN questions using Rust's exact native lowercase and
alphanumeric-token functions. Exclude the existing every-tenth CAL indices
0,10,20,... and every remaining row whose normalized question equals any CAL
question. Record all exclusions, order and hashes. Do not inspect new CAL labels
or generate new CAL predictions. Remaining rows are FIT; keep duplicates within
FIT but report normalized-question groups and never later split a group across
training/validation. Runtime extraction receives only these question strings and
the read-only graph. Labels are joined offline after outputs are complete/hashed.

For each question enumerate every contiguous normalized token span of at least
two UTF-8 bytes, with token and byte boundaries. Query the existing alias lookup
for up to two distinct entity IDs using exactly native acronym support semantics.
Preserve ambiguous spans as ambiguous; do not resolve them through annotation IDs,
claim existence or relation labels. Mark unique, case-supported spans as eligible.
Repeated occurrences retain their separate positions. Normalization is copied
literally from the native implementation for this research example and version
checked against embedded graph metadata; production code remains unchanged.

Reconstruct the existing longest-alias result independently from the candidates
and require equality with DiskStore.link for EVERY FIT question. Both casing-
unsupported and ambiguous longest matches must block shorter fallback, exactly as
native code does. The study then reports whether any eligible span identifies
the gold entity, whether baseline is right/wrong/unlinked, gold-span multiplicity,
and coverage by normalized-question group. Entity IDs/answer IDs may be diagnostic
labels only, never learned features or runtime answer tables.

Before real extraction, authored contracts must cover longer nongold aliases,
equal-length ambiguity, lowercase acronym blocking shorter fallback, repeated
mentions, non-ASCII punctuation/lowercase and normalized offsets. All must match
native link results. Question limits stay4096bytes/128tokens. The research
extractor retains at most512 matched span occurrences per question; exceeding
this or a SQLite query budget fails preparation without row dropping or retry.
Read-only SQLite cache64MiB, no mmap, query-only transaction,10million VM operations
per question. A separate native parity call has the unchanged native budget.

Run preparation once under1800seconds,8GiB/no swap,2GiB added-run cap and private
on-disk TMPDIR/SQLITE_TMPDIR. Compiler builds separately bounded at600seconds,
at most4cargo jobs, same8GiB/no-swap scope. No model workers or credentials needed.
Pin scripts/binary/input hashes, retain original outputs and any failure. Require
all rows, exact parity, all contracts and resource gates. Only justify subsequent
learning if there are at least500 unique FIT question groups with an eligible
correct span and at least100 unique groups where that correct candidate is missed
by the current linker. Otherwise change the representation rather than fit an
under-supported model. Fitting requires a separate frozen design after this
coverage report; this preparation cannot promote any runtime change.
