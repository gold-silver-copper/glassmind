# Normalized search rejected by the frozen improvement gate

The independent parent scorer confirms LAS19,423→19,428 (+5) and UAS20,540→20,547
(+7) over25,148 words/all2,001 EWT DEV sentences. The frozen requirement was at least
19,673 LAS with no UAS loss. All trees are valid and there are no refusals, but the
accuracy gain is too small for this experiment's acceptance gate. No runtime or
model changes are integrated. Full corpus includes68 TRAIN-duplicate sentences;
all137 duplicate edges remain correct. The corpus gate was not adjusted afterward.

The parent froze20 fresh sentences/40 core labeled edges before candidate inspection.
Baseline and candidate both score36/40 edges and16/20 complete; no edge changed its
correctness. This independent probe passes, but cannot override the failed corpus
criterion. The authored24 cases/72 edges were the worker's own evidence.

Verified73 original rejected archive entries/61,209,688bytes;32 parent snapshot
entries/121,136bytes;55 raw identities/166,885,684bytes;27 frozen identities,
including original source copies retained before rollback. Frozen paths to the two
now-restored runtime files were checked against candidate/syntax.rs and standalone.rs;
all source hashes agree. Both current runtime files equal the root approved source.
The initial verifier assumed a list-shaped manifest; it was corrected to read the
actual mapping. No archive, result or model was modified.

Original reports, full25,148-edge/2,001-sentence transitions and binary remain under
/home/gsc/.cache/glassmind/continuous-20261010/syntaxnorm-run. Parent audits are in
coordinator/syntaxnorm-review and syntaxnorm-independent. Default forms/raw outputs
and fixedmodel47d8d245 remain byte-identical. Candidate binaryb15f1a0a...; full output
e008c7dd95ae8a06835a5afc2bc7688ba36bf10435d42f97a7ddab6108a338c2.
No new downloads or held-out access. A future parser experiment must change training
or features; neither rejected search objective warrants another width/score sweep.
