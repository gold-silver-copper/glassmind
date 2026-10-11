# QA syntax v3 — passes frozen gates; optional unverified mode only

One candidate on base `078578b0ac350a1b3baa5af20b1a37d21d576d23`.
No commit or main-command integration. Runtime changes remain uncommitted for
coordinator review. No replacement of approved v1 model. No agents, LLM calls,
services, API keys, credentials, downloads, new corpora, held-out data, or syntax
retraining. All owned native/build jobs finished. Parent independent replay and
explicit optional main integration remain outstanding.

## Changes

- `crates/reading/src/qa.rs`: only a public `syntax_v3` child-module declaration.
  Existing v1 definitions, inference and serialization are unchanged.
- New `qa/syntax_v3.rs`: reusable `Companion`, `SyntaxModel`, `Session` API.
  Version3 wraps the unchanged65536-weight v1 linear ranker structure; five
  averaged passive-aggressive passes, rate cap0.1. Adds predicted head/parent
  UPOS/deprels, bounded ancestor paths to matched question anchors, question-role
  and attachment features, auxiliary/subject/object child signals and negation
  features. Same top3 sentences/max10 QA-token spans and original input limits.
  Raw Unicode parser tokens align by byte boundaries. Missing/over128 parses and
  unaligned candidates have explicit missing features; unknown words stay parser
  predictions. Parses are cached with a16384-text cap; dependency feature vectors
  are computed per possible head per question/evidence sentence, then reused
  across spans. These signals are learned ranking inputs, never entailment rules.
- New `reading-qa-syntax` trainer/predictor. Required `--syntax` companion; input
  permits only context/question. Output retains exact UTF8 proposal/evidence and
  adds `semantic_status: "unverified"`; selected/refused remain policy states.
  Library predictions use the existing `Prediction` shape and unverified reason;
  parent integration must preserve the adapter's explicit semantic status.
- Crate-local `sha2=0.10.9` and one Cargo.lock dependency edge; sha2 was already
  locked/cached, no download. Exact approved syntax SHA256 is checked before its
  bounded validated loader. Model version and companion identity are checked;
  absent/incompatible pairs fail closed. No path-based identity or fallback.
- New ownership-local adapters, preregistration, authored probes and reports in
  this directory. No frozen syntax, CLI, English/world, scorer, shared manifest,
  README, historical artifact or evaluation definition changed.

## Frozen experiment and result

[PREREGISTER.md](PREREGISTER.md) preceded implementation/fitting/DEV. Only first16
FIT examples were inspected; train aggregates retained. All11 supplied manifest
entries and all4 source identities verified. Reconstructed FIT4000/64article and
CAL1000/next16article files plus selection.json independently; all five byte-equal
originals. Every epoch3214/4000 representable, unchanged from v1. One fit only.
Prior support/relative-anchor experiments remained rejected, with gates unchanged.

Original CAL rule: maximum coverage with >=50% normalized exact and>=30 accepts,
including complete margin ties. Selected166,83right83wrong at margin
`0.20622891187667847`. Full frontier external. Independent group-wise recomputation
agrees. No probability claim; no refit, threshold lowering or post-DEV retuning.
Source/model/companion/scorer/data/probes frozen before first200 SHA256(ID), then
all10570 once. Unmodified official functions imported via existing qa/run.py;
pinned scorer SHA checked and every original question retained.

| Metric, all10570 | Approved v1 | Optional v3 |
|---|---:|---:|
| Raw exact count |2532|3057|
| Raw exact % |23.9545884579|28.9214758751|
| Raw F1 % |34.5184250480|39.8695725750|
| Selected correct |815|943|
| Selected wrong |791|647|
| Refused |8964|8980|
| Selected precision % |50.7471980075|59.3081761006|
| Coverage % |15.1939451277|15.0425733207|
| Selected F1 % |8.7001900898|9.7669285748|

Raw gain525 exact/+4.9668874172 percentage points and+5.3511475270 F1.
All four aggregate gates pass:3057>=2744,39.86957>=36.5184250480,943>=815,647<=791.
Fixed first200: raw46->60 exact, F131.2087565->42.5411616; selected16/17/167
right/wrong/refused ->21/10/169. No parameter change followed the sample.
[Full](reports/full.json), [sample](reports/sample.json),
[acceptance audit](reports/acceptance.json), [freeze](reports/freeze.json).

## Losses and every transition

An aggregate gate is NOT per-item semantic preservation. Raw720 formerly correct
proposals become wrong;1245 formerly wrong become right. Raw transitions:
right->right1812, right->wrong720, wrong->right1245, wrong->wrong6793.

| Selected before -> after | Count |
|---|---:|
|right -> right|477|
|right -> wrong|4|
|right -> refused|334|
|wrong -> right|14|
|wrong -> wrong|167|
|wrong -> refused|610|
|refused -> right|452|
|refused -> wrong|476|
|refused -> refused|8036|

480 newly wrong selections come from old right/refused states;338 old correct
selections are lost;466 correct selections are gained.15 wrong->wrong selections
also change text.6689 raw strings and1905 selected strings change. All10570 IDs,
old/new raw and selected text, and both state transitions are in external
`full.transitions.json` (2,411,027bytes), including unchanged cases. Every200-row
transition also retained. Refusals are classified by status BEFORE exactness;
normalization-to-empty gold variants never make a refusal substantively correct.
The corrected totals agree with815 old and943 new substantive correct selections.

## Authored contrasts and known semantic failures

Twenty probes frozen BEFORE training/DEV. First8 straightforward positive role
pairs are8/8 raw correct and8/8 selected correct (subject/object reversal, passive,
multi-clause distractor and Unicode); v1 had6/8 raw and4/8 selected. All14 positive
cases have correct v3 raw answers and selections. No new selection on the two
clear predicate/negation mismatch cases relative to v1. Hence both frozen
contrast gates pass. All20 raw/selected outputs and expected behavior are in
[probes](reports/probes.json). This is a small authored diagnostic, not a broad
independent benchmark or entailment guarantee.

**Inherited selected failures:** painted bridge vs question asking when demolished,
and never-demolished bridge vs positive demolished question BOTH still select2008.
Nothing here establishes negation or predicate entailment. Unsupported count,
unknown entity, ambiguous entity and place mismatch cases refuse, but their raw
proposals remain wrong or unsupported; abstention is not understanding. Six
negative cases have no unique substantive gold span, so raw EM is not awarded
on them. Every raw mismatch is retained instead of being scored as correct refusal.

After DEV,39 existing public authored probes (roles18/support15/originalQA6) were
replayed as **unscored diagnostics**, with duplicates retained and no new gates or
retuning. [All legacy results](reports/legacy-probes.json) record12 v3 selected
failures versus14 v1 failures. **Two distinct newly selected unsupported cases**:

- `Felix thanked Nora.` / `Who did Nora thank?`: v1 refused, v3 selects Felix.
  Appears twice across historical probe sources. This is a reversed-role failure,
  despite the8 correct straightforward positives.
- `Tessa counted 2017 stones.` / `Where did Tessa count stones?`: v1 refused,
  v3 selects2017. Count/place mismatch remains a substantive limitation.

Other legacy failures: unsupported reversed-subject question (two occurrences),
painted/never-demolished predicates (four), person answer includes `Soren in Oslo`,
unsupported buy question selects `owns`, and unsupported bicycle-buy question
selects `painted`. All12 are retained with v1/v3 raw spans and selected behavior.
Original approved parser errors (Who did Nora thank? role, Birds fly, negation,
attachment, contractions) are not repaired: parser bytes unchanged. We did not
rerun EWT or claim a new parsing score. Original parent evidence reports77.23% LAS
on gold-tokenized EWT; raw Unicode passage/question parsing has no such guarantee.

## Validation and artifact identity

- Release build and crate fmt check pass; strict clippy passes for lib and both
  QA binaries. Initial clippy caught one JSON object indexing expression in the
  NEW adapter. Replaced with checked object insertion before CAL/DEV; feature
  source/optimizer/model unchanged. Initial log and pre-fit hashes preserved.
-12 library tests pass:6 original QA,4 original syntax,2 new pair/finite/Unicode
  tests.2 original byte-probability tests pass. No giant/workspace suite.
-10 external contracts pass: reject IDs/titles/answers/heads/forms, missing/wrong
  companions and model-pair mismatch; UTF8/provenance/input bounds/missing-parser
  handling, unverified status and determinism. Extra finite-max-weight model
  actually exercises score overflow and returns finite refusal.
- Full10570 ordinary v1 replay byte-identical to supplied approved output:
  `37db70ae0c949a89427a707c37812a666d9507a2192bcb75ed3e4145d3a598a7`.
  Existing Model fields/implementation/serialization unchanged. No automatic v1
  replacement. Frozen syntax source and binary implementation untouched.

External artifacts:
`RUN=/home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run/experiment`.
Contains model, copied evaluated binaries/source, FIT/CAL, full inputs/predictions,
transition ledgers, complete CAL frontier, all build/test/evaluation logs. Runtime
patch also archived before handoff. [provenance](reports/provenance.json) hashes
all external artifacts plus new repository files and records original sources.

| Artifact | Bytes | SHA256 |
|---|---:|---|
|v3 model|1078136|`7cd2afb5fbb726f2bf09d98e54ae0e98807b178a7e49dcf85513be1f396315ad`|
|required syntax model|12554724|`47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6`|
|evaluated v3 binary|13031656|`b0ad5d9655fe3f4b97804150446bca60761f8d406028be131424b398aa17c277`|
|full v3 output|2472470|`7580e7b9cb4b4ec5a845a7f68b8631d03bd3011480b9cf8e82f8f7de55e28277`|
|full transition ledger|2411027|`2a7b1fbaa8418325268696e2f72e504b38d96c9762f3a7322a1d5119d812b09c`|

## Sources, licenses and reproduction

Zero new external data bytes or downloads. Reused SQuAD1.1 TRAIN30,288,272bytes,
DEV4,854,279bytes (35,142,551total), revision
`eee5fdbf62f8613a7812b03419e6b29617b74fd1`, corpus CC-BY-SA-4.0;
repository/scorer MIT. Source: https://github.com/rajpurkar/SQuAD-explorer/tree/eee5fdbf62f8613a7812b03419e6b29617b74fd1
Dataset attribution: https://rajpurkar.github.io/SQuAD-explorer/ . Derived FIT/CAL
and inspected quotations retain corpus attribution/license. Original FIT train
3,573,508bytes, FIT input3,423,180bytes, CAL input869,071bytes, CAL labels1,029,325bytes.
Twenty original authored probes4,120bytes, MIT; no generated/held-out test material.
Code MIT. New dependency sha2 0.10.9 MIT OR Apache-2.0, already present in lock/cache.

Reused syntax model trained only on UD English EWT TRAIN, r2.18 revision
`b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c`, CC-BY-SA-4.0;
TRAIN15,034,493bytes according to frozen source provenance. No EWT items read,
new data downloaded or syntax refit this batch. Source:
https://github.com/UniversalDependencies/UD_English-EWT/tree/b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c .

For reproduction use a disposable checkout because adapters regenerate the owned
reports, a fresh RUN beside original baseline, and the existing target directory:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run/target
export SQUAD_SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-source
export QA_SYNTAX_RUN=/home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run/replay
bash tools/reading/qa-syntax/reproduce.sh
```

Actual stages ran individually, not a second convenience-script run. Every cargo
invocation set persistent TMPDIR and ran under `systemd-run --user --scope
-pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`, with
-j1 except fmt, tests --test-threads4. All native training/inference/evaluation
also scoped; outer scope verified8GiB/zero swap. No jobs outlive worker. No /tmp
storage or broad process kills. Reproduction script contains exact commands.

Remaining work: coordinator independent replay, risk review of individual
regressions, and optional main integration if accepted. This batch provides a
measured optional prediction improvement, not trusted facts, probability
calibration, semantic preservation or general reading comprehension.
