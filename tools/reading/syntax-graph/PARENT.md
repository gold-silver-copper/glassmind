# Graph parser rejected after independent review

The unchanged independent scorer confirms full2001 DEV LAS19,423→17,900 and
UAS20,540→19,732 over25,148 integer words including punctuation. Both frozen
corpus gates fail. Every tree is structurally valid; that does not establish
linguistic accuracy. All68 TRAIN-duplicate sentences/137 words remain included
and correct. The parent independently recomputed every edge/tree correctness
transition and matched the worker's complete aggregate counts.

The parent froze24 fresh sentences/52 core labeled edges before candidate
inspection. Baseline46/52 and19/24 complete falls to41/52 and15/24 complete, in
both provided-forms and raw-text modes. Seven old correct edges are lost and two
gained. All byte spans and tree structures remain valid. The core preservation
gate also fails; no expectation or denominator was changed.

Twelve independent score matrices with two to five tokens have exhaustively
enumerated maximum single-root tree scores. A native wrapper around the exact
frozen decoder function bytes matches every optimum, including negative weights
and ties. This supports decoder correctness on those bounded inputs, not model
accuracy or a proof over all graphs. No training was repeated.

Original terminal archive:206 files/93,888,678 bytes, verified before inspection.
Frozen worker archive:97 files/60,767,507 bytes, independently verified. All27
pre-DEV frozen identities agree, including archived runtime source. The approved
and candidate taggers share the exact1,328,492-byte serialized payload,
SHA0409e47e9376ee1593480461fa35bbe7cfa1c21f6d74fce7ba9bad167dbba39f.
Retained legacy forms/raw predictions are byte-identical. Both restored runtime
files equal the root checkout; the new graph module is absent from production.

Source review confirms labeled arc maxima, exact single-root constraints, checked
finite scores, unchanged predicted tagger inputs, three ordered structured
perceptron epochs and averaged updates. The trained model is9,715,189 bytes.
This rejects that fixed feature/objective/training configuration; it does not
establish that all graph parsers would regress. No model or runtime code is
integrated. Avoid tuning this configuration on these observed DEV failures.

Five parent evidence-adapter path failures were corrected without modifying any
candidate: the legacy prediction filename, an archive-prefix collision, and
shared scorer/measurement file resolution and scope checks. Original scripts
remain in the external parent folder.
The subsequent complete evidence audit passes. Original worker clippy failures
and all model/training limitations remain in HANDOFF.md.

Compact contracts, independent scores, transitions audit and decoder probe are in
`parent/`. Raw evidence remains under
`/home/gsc/.cache/glassmind/continuous-20261010/graphsyntax-run` and
`coordinator/graphsyntax-independent`. EWT r2.18 b7711cce, CC-BY-SA-4.0, was reused;
no new corpus download or official TEST access. Worker usage2,094,923 tokens
includes cached input once; parent usage is excluded.
