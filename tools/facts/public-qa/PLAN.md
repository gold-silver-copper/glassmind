# Fixed public-pilot QA coverage experiment

Frozen before any benchmark question is run against the public native graph.
The preceding storage, raw-source and query API gates passed. This experiment
tests whether that independent coverage improves actual annotated QA, keeping
the relation model and query rules unchanged. It does not automatically replace
a default bundle or combine this graph with a QA-selected index.

Inputs:
- Public native database: 12,306,595,840 bytes, SHA256
  f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b.
  First of 86 lexical claim shards plus all en/mul names; CC0 Wikidata 20260928,
  publisher permutans, scholarly works excluded. No QA-based graph selection.
- Existing relation model: SHA256
  db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6.
  No training, threshold search, alias edit or normalization change.
- Integrated native CLI: SHA256
  d733e1d9f19e5e4e5a03087f9d12725fed1b212f5a1b3b47f4573a7ac28d4a3d,
  source reviewed in PR49 at9e5bdcc; copy and pin it before inference.
- Approved SimpleQuestions TRAIN: 19,481 rows, SHA256
  25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b.
  askplatypus/wikidata-simplequestions revision
  df69267f1db4f85a71325e7e8c06e192bcaf0b0e, CC BY3.0.
- Paired reference is the frozen TRAIN-selected 999-entity graph, not the
  validation-subject-seeded historical 500-entity series. Its full TRAIN
  predictions have SHA256
  eb09e5fc44e699e2ab08b94a44409ab8ad5a623224bd7340372c4326bcb0aee8.

Use the existing every-tenth, zero-based TRAIN calibration partition: indices
0,10,20,..., for exactly 1,949 rows. Validate the complete TRAIN question order
against the retained question file before extracting this partition. Inference
gets only question text, model and read-only graph. Freeze complete predictions
and their hash before the separate annotation join. Never read official held-out
items, labels, predictions or generators. This CAL partition is development data,
has been used in earlier experiments, and is not an independent generalization
test of the pre-existing relation model.

Primary metric: exact annotated entity, directed P/R relation and sole answer,
using the unchanged joint-audit.audit function. Also report the unchanged
answer-only metric, correct-link counts, refusals, all nine paired status
transitions, lost correct answers and newly wrong answers. Keep all 1,949 rows;
no redirect rewriting, annotation repairs or reinterpretation after results.

Prospective promotion prerequisite: at least ten net additional strict joint
correct answers on CAL, with zero baseline-right or baseline-refused cases
becoming wrong. Report every baseline-right loss, including safe withdrawals.
Do not weaken the criterion after results. A failure leaves the public graph an
explicit storage artifact and does not promote it as a default QA replacement.

Only if CAL passes, run the complete approved 2,821-row DEV with the same frozen
graph/model/CLI and no intervening tuning. Require at least ten net strict joint
correct gains over the retained TRAIN999 DEV baseline and zero newly wrong
baseline-right/refused cases there as well. DEV is development evidence, not
held-out accuracy. Keep its series separate from historical answer-only scores.

Regardless of CAL outcome, diagnose CAL coverage using annotations only offline:
anchor presence, actual link correctness, and the annotated relation's local
claim category (missing, unique gold, nonunique including gold, or gold absent).
These are oracle knowledge diagnostics, never predictions. They must not supply
triples, source file selection or runtime features. If coverage is inadequate,
choose a new representation or independently justified acquisition experiment;
do not silently fetch QA-selected entities or the remaining 85 shards.

Bound the complete CAL run/diagnostic to600 seconds under8GiB/no-swap, with a
120-second inference timeout and on-disk TMPDIR. Added run files must remain
below256MiB, excluding the already-existing input graph/model/data. Preserve any
failure and partial output. No downloads, external model calls or model workers.
