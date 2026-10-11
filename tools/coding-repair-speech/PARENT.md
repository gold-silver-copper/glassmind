# Original repair-request candidate fails independent coverage

A subsequent coordinator revision restores8/8 positive repairs, blocks all9 old
unrequested edits and passes15 legacy native repairs. Full765 is running; this is
not yet a published improvement. See [revision validation](parent/revision/VALIDATION.md).
The original rejected submission and its scores below remain unchanged.

The original submission removes all nine unrequested do edits from the parent
baseline, but loses four of eight previously working repair requests. It answers
only four of six status forms under the frozen proxy. No production runtime is
integrated and no full765 is warranted on this candidate. The published routing
bug remains open while a new revision is prepared.

The parent froze30 cases before inspecting the candidate:8 explicit repairs,
6 status questions and16 unsupported/non-request inputs. Each runs through initial
ask --apply and do on fresh broken fixtures. Baseline do performs8 exact repairs
and9 unrequested edits; candidate do performs4 exact repairs andzero unrequested
edits. Negative previews expose no Command. Every applied repair passes the
original assertion. ask --apply correctly retains Remedy confirmation for both
binaries; it must not be changed to auto-apply those proposals.

The initial parent summary mistakenly required both modes to mutate and reported
zero positive repairs. It is preserved beside the corrected summary and contract
note, written before candidate review. Inputs, expected labels and original outputs
were unchanged. Parent native targets are fixture-owned, with CARGO_TARGET_DIR
unset; the candidate replay explicitly clears that variable. The worker's separately
reported earlier shared-target interference does not invalidate these parent runs.

The worker's original native expectation remains180/204, with24 initial-apply
expectation failures; supplied/legacy expectation failures also remain. Separate
explicit confirmation checks pass2/2. Those reports are not relabelled as all-pass.
The supplied14 diagnostic inputs were coordinator-authored, not an external dataset.
Original source, binary, patch, failed attempts and reports are retained without
modification. Only the final frozen binary is used for this independent review.

Verified:66 parent archive entries288,971bytes;67 original archive entries288,971bytes;
85 worker source/result identities341,035,550bytes. Original debug candidate SHA256:
`f8699eb506931014ce833046d86018f18b5cfc0f0c06f708518d2816e705264f`.
Raw artifacts: `/home/gsc/.cache/glassmind/continuous-20261010/repairspeech-run`;
original submitted and parent-submitted snapshots are separate. The inactive
worker checkout remains available for a coordinator revision; it has not been
modified or removed. No new data, downloads, model weights or backend changes.

The missing positive families are singular diagnostics, semicolon-delimited failure
context, explicit behavior-preservation clauses and Rust-qualified source nouns.
Any revision must consume these families without accepting unknown conditions or
additional operations. [Revision requirements](parent/REVISION.md) also flag the
status proxy: Problems(Build) is a legitimate public check query alongside Passes.
Do not introduce incorrect polarity to satisfy a test. Preserve/rebaseline any
legitimate evaluator clarification explicitly; the four lost positives reject this
candidate independently of that issue. A revised binary needs the unchanged native
positive/refusal gates and all180 published coding passes before publication.
