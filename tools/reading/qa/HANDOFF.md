# Reading QA: one bounded batch, accepted under the frozen experimental gate

Uncommitted implementation for coordinator integration. Base source
`b36970d874c1d2b54c2fc31436782bd8c6337332`. No main command integration,
shared manifests, existing reading paths, scorer files, other tracks, or historical
prompts changed. No agents, additional LLM calls, credentials, downloads, services,
background jobs left running, or commits. Existing session model/authentication was
not changed or independently inspected; no new model/authentication calls occurred.

## Outcome and frozen acceptance

[PLAN.md](PLAN.md) was written before implementation/evaluation. One candidate only;
no dev-driven feature, training, threshold, or sampling changes. Gate: >=2 percentage
points raw EM gain, >=3 F1 gain, >=1% dev accepted coverage and >=40% accepted exact
precision. Candidate **passes**. This is acceptance of a measured extractive QA
experiment, not a claim of verified answers or reliable general abstention.

Official normalized metrics, all 10,570 dev questions retained:

| System | Raw EM | Raw F1 | Accepted EM | Accepted F1 | Exact / refused / wrong |
|---|---:|---:|---:|---:|---:|
| Fixed lexical extraction baseline | 12.393567 | 19.001517 | 12.346263 | 18.945925 | 1305 / 108 / 9157 |
| Learned span ranker + TRAIN threshold | 23.954588 | 34.518425 | 7.710501 | 8.700190 | 815 / 8964 / 791 |

Raw gain: **+11.561022 EM / +15.516908 F1 percentage points**. Raw exact counts:
1310 -> 2532. Candidate accepts 1606/10570, coverage **15.193945%**;
exact precision **815/1606 = 50.747198%**, with **791 wrong**. Baseline coverage
10462/10570 =98.978240%, exact precision 1305/10462 =12.473714%.
Raw proposals include refused answers; accepted metrics use blank strings for
refusals. These are distinct operating points, not a claim of higher accepted EM.

Frozen SHA256(questionID)-ordered sample 200: baseline raw13.0 EM/18.813095 F1;
candidate23.0 EM/31.208757 F1; candidate16 exact/167 refused/17 wrong, precision
16/33=48.484848%, coverage 33/200=16.5%. Sample preceded full evaluation.
[Full](reports/full.json), [sample](reports/sample.json), [audit](reports/audit.json).

Regression transitions are not hidden: baseline accepted right 1305 ->262 still
right,976 refused,**67 accepted wrong**. Candidate adds 553 accepted right from
baseline wrong. All 108 baseline refusals remain refusals. Raw baseline right 1310
->669 right/641 wrong;1863 formerly raw-wrong become raw-right. No claim that all
baseline correct cases are preserved.

## Implementation and API

- `crates/reading/src/lib.rs`, `qa.rs`: public `Model`, `Input`, `TrainingRow`,
  `Span`, `Prediction`, `Status`, `baseline`. `Model::predict(context, question)`
  returns only source-extracted text; half-open UTF-8 **byte** offsets identify the
  proposal and enclosing sentence evidence. `Span::text` checks boundaries.
- `reading-qa` separate Rust binary: offline `train`; strict JSONL `predict`.
  Inference accepts exactly `context` and `question` (`deny_unknown_fields`);
  IDs/titles/gold/answer_start never enter runtime. JSON output reports
  `predicted`/`refused`, blank accepted answer on refusal, retained raw proposal,
  score, margin, and reason. Scores/margins are explicitly not probabilities.
- General templates: question-content overlap, nearby overlap, capitalization,
  numbers, lengths, sentence position, punctuation and lexical boundary priors,
  crossed with question first word/content head. No entity-specific branches,
  IDs, title features, answer tables, regex answer rules, pretrained networks,
  embeddings, runtime LLMs or network access. Feature hashing uses fixed FNV-1a.
- Top 3 sentences by unique question-content-word overlap; all 1–10 word-token
  spans within them; max 65536 context bytes, 2048 context tokens,2048 question
  bytes. Input-limit or no-content-question cases refuse. Zero lexical support
  refuses even if a raw proposal exists. Duplicate answer strings do not compete
  against themselves for margin; first offset wins score ties.
- 65536 float32 weights; 34827 nonzero;262144 dense weight bytes. Five ordered fit
  passes; passive-aggressive pair updates, unit margin, maximum step 0.1; lazy
  averaged weights. Existing workspace classifiers target fixed classes rather
  than variable source spans, so no new crate dependency/trainer coupling.
- `run.py` owns dataset IDs, labels, conversion of TRAIN character offsets to
  UTF-8 bytes, calibration, and scoring. Before invoking official functions it
  asserts output count/unique exact ID sets; full prediction maps contain exactly
  every 10570 dev ID, including blanks. No skipped or removed examples.

## Frozen training/calibration

TRAIN is 87599 questions/442 articles. Article hash is SHA256(newline-joined
contexts), ascending; first 64 articles fit, next 16 calibration, disjoint. Selected
article pools contain 11196 and2345 questions. Within each pool, ascending
SHA256(questionID), first 4000 fit and 1000 calibration. Full article hashes and IDs
are frozen in [selection.json](reports/selection.json). First annotated TRAIN
answer supplies the fit span.3214/4000 fit rows have an exact proposed span;
786 are unrepresentable and produce no rank update, still disclosed in fit count.

Calibration uses no dev labels: select maximum coverage among distinct margin
thresholds with >=30 accepts and >=50% normalized exact precision on 1000 TRAIN
calibration questions, grouping ties. Threshold **0.18979406** accepts 148:
74 exact,74 wrong,852 refused (precision 50%,coverage 14.8%). If none qualified the
frozen rule required all-refusal and candidate rejection.
[Calibration](reports/calibration.json). This selection-set estimate is not an
independent confidence calibration. SQuAD1.1 has no genuine unanswerables.

## Validation, failures and limitations

Scoped `cargo fmt --check`, strict all-target clippy, release build, all-target
reading tests: **5 new library tests +2 existing probability tests pass**. Tests
exercise UTF-8 offsets, bounds/no-support refusal, metadata rejection, repeated
occurrences, training/serialization and invalid-model refusal. Logs in `reports/`.
Independent fit repeat yields identical weights/counts. Final rebuild after a
clippy-only `if let` collapse yields byte-identical sample/full baseline/candidate
outputs and unchanged model; `attest.py` verifies this against retained hashes.

[Independent probes](reports/independent.json): Unicode Élodie, no-overlap refusal,
ambiguous Ana/Bea refusal, repeated Ana occurrence, and date extraction with
unrelated trailing text behave as intended (5/6). **Unsupported relation fails**:
“Lena painted a bicycle.” / “When did Lena buy the bicycle?” accepts “painted”.
This is retained evidence, not patched after evaluation. Source occurrence is
provenance, not proof of correctness. Negation, entailment, relations, ambiguous
names, abbreviations/decimal sentence boundaries, and long answers are unresolved.
The accepted error rate remains 49.25%; this API must remain explicitly predictive.
No claim of general unanswerable calibration or production-safe autonomous answers.

No alternative model experiments were attempted/rejected; this one passes its
predeclared gate. A strong general semantic-support claim is rejected by the above
probe. Infrastructure corrections: initial source-directory parent typo; absent
`/usr/bin/time` (failed before training, rerun without it); one clippy style error.
No scoring definition, denominator, feature, threshold, or fit-selection adjustment
was made in response to dev results. Resource peak was not instrumented; all
jobs remained inside 8GiB/zero-swap scopes, with 600s timeouts for cargo/fit/evaluation.

Original `main.rs`, `probability.rs`, `reading-probability.rs`, crate/root Cargo
manifests and Cargo.lock are byte-identical to base, verified in provenance.
Parsing/byte probability/BLiMP code paths and shared grammars/model sources are
unchanged. Original full byte/BLiMP datasets were **not rerun**; scoped existing
probability tests pass. No full-workspace or held-out evaluation was performed.

## Sources, licensing and exact identities

SQuAD1.1 TRAIN/DEV from `rajpurkar/SQuAD-explorer`, pinned
`eee5fdbf62f8613a7812b03419e6b29617b74fd1`, CC-BY-SA-4.0. Attribution and license
links: [LICENSES.md](LICENSES.md). No additional data was fetched (0 new download
bytes). Local readonly source `/home/gsc/.cache/glassmind/continuous-20261010/readingqa-source`:

| File | Bytes | SHA256 |
|---|---:|---|
| train-v1.1.json |30288272|3527663986b8295af4f7fcdff1ba1ff3f72d07d61a20f487cb238a6ef92fd955|
| dev-v1.1.json |4854279|95aa6a52d5d6a735563366753ca50492a658031da74f301ac5238b03966972c9|
| evaluate-v2.0.py |10547|710840ce2c2c61716334c056777032f24660b83eb9869a451036e6a24d8103c4|
| source LICENSE |1073|9c92e90621a47dc8508ddd78f0ee41a7b6a8fd5d37fd00709ff1c6eb1a7d7019|

Official scorer file unchanged. NumPy is unavailable, so the adapter compiles the
**unchanged AST bodies** of its normalize_answer/get_tokens/compute_exact/compute_f1/
get_raw_scores/make_eval_dict functions after asserting the pinned file hash.
Only standard-library imports are supplied. No no-answer probabilities, CLI
threshold machinery, altered normalization, or substitute scoring functions.

New code MIT; dataset-derived weights/selections/results CC-BY-SA-4.0.
Model `artifacts/model.json`: **873881 bytes**,
SHA256 `b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5`.
Final binary **11339944 bytes** at
`/home/gsc/.cache/glassmind/continuous-20261010/readingqa-run/target/release/reading-qa`,
SHA256 `5ee4b058820acc23e9bf7b3a96f8d85e5e0b62faee921585eb95820bb408bfa7`.
Source manifest SHA256
`6c926391f0d17d68fa7bfc335ae816fdd02cf5b5e350a5d9582a9aaa7d83b6ee`.
[provenance.json](reports/provenance.json) contains per-source-file and all generated
artifact byte counts/hashes, compiler identity, original dataset provenance and
unchanged-path hashes. The source manifest excludes this handoff and generated
reports, avoiding circular hashing. All model/selection/prediction identities are
retained. Non-model `artifacts/` files are intentionally git-ignored but retained
locally for coordinator archival; they are reproducible from the pinned sources.

## Reproduction and integration

From repository root (no network or API keys):

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-run/target
export SQUAD_SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-source
export QA_TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-run/tmp/qa
bash tools/reading/qa/reproduce.sh
```

Script wraps **every cargo call**, training and evaluation in
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`; TMPDIR is an on-disk directory, cargo uses -j1 (except fmt),
and tests use `--test-threads 4`. Runs prepare, fit, TRAIN calibration, fixed sample,
full dev, independent probes, and attestation in that order. Original selected
model and prediction digests serve as reproducibility assertions. Fit labels,
calibration labels, and dev IDs live only in offline adapter artifacts.

To use an existing model without training:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-run/tmp/qa
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 \
  "$CARGO_TARGET_DIR/release/reading-qa" predict \
  --model tools/reading/qa/artifacts/model.json --input context-question.jsonl
```

Each input line is `{"context":"...","question":"..."}`. Rust callers deserialize
`qa::Model`, call `validate()`, then `predict(context, question)`. Coordinator can
integrate that library API into the main command later. This batch is finished;
no incomplete implementation or running jobs remain. Broader semantic support,
better precision and independent unanswerable evaluation remain future work.
