# Learned system-offer span selection — bounded candidate

Base source: `da96f494d4a6865924cb0aa5eea138b029158fc2`. Uncommitted changes only
in `crates/dialogue/` and new `tools/dialogue/offers/`. No coordinator/shared
files, main CLI, scorer, evaluator, manifest, README, or historical prompt edited.

**Accepted under the preregistered dev criteria:** joint turns **31→35/91** on
the fixed sample and **595→609/1119** full; multi-service RideSharing exact
states **0→23/265**. This is actual slot-state improvement. All routing outputs,
router/service/reference weights, and all 595 baseline-right turns are retained.
The baseline is NOT error-free: its **four erroneous copy events persisting
for 11 scored slot-turns remain unchanged**. New offer additions introduce zero
wrong claims in this declared dev selection; no broad error-free claim is made.
Coordinator integration source is `ba98b5a56cef82ba57357256775e7b461fbcf017`.
Independent training reproduces the model bytes; the integrated binary reproduces
full predictions and legacy predictions exactly. Frozen baseline replay and
179-conversation isolation pass. Workspace fmt/clippy/build/strict4/4 smoke and
six focused tests pass. See `reports/integrated/validation.json`. The worker
made no commit or publishing action.

## Hypotheses and training-only decisions

[EXPERIMENT.md](EXPERIMENT.md) was written before implementation/dev scoring.
Both hypotheses kept sample/full/legacy acceptance and frozen baseline weights.
No dev calibration, changed scoring, ignored frames, or second dev-tuned setting.

H1 learned paired 1–3-token left/right context boundaries around exact TRAIN
state values appearing in the immediately preceding SYSTEM utterance. All
annotated selected-service USER frames supplied opportunities, including absent
slots. No gold slot spans, actions, API results, service lists or gold states
enter inference. Initial unconstrained spans absorbed time expressions, articles,
and ride qualifiers. **Rejected before dev:** 3659 proposed patterns, 169 with
minimum evidence, zero passing rules. Complete rejected counts are retained in
`reports/rejected-h1-training-report.json`; no H1 dev gain is claimed.

H2, frozen before any dev evaluation, adds a conservative source-boundary shape:
first token uppercase/numeric; following words uppercase/numeric or generic
connectors; bounded internal punctuation. No city, attraction, service ID, or
answer-specific branch. Exact spelling/case/bytes remain the source's.

SHA256(dialogue_id) mod 5 == 0 divides the 3174 training conversations into
**2532 fit / 642 calibration**. Each retained rule needs >=5 correct distinct
source turns in fit, >=5 distinct aligned training values, >=3 correct source
turns in calibration, and **zero incorrect proposed spans in both**. These are
empirical selection counts, not calibrated probability guarantees. Context rules
are proposed on fit only. Calibration neither supplies entities nor new rules.
The final inspectable model has **26 rules: 25 for destination and 1 for city**;
these target names are learned/schema-validated data, not hardcoded runtime IDs.
All 11 actual new dev selections are destinations; no benefit is attributed to
the learned city rule. Boundary context and counts contain no stored entity
values. `reports/offers.json` is 8132 bytes, CC-BY-SA-4.0 derived weights.

## Exact implementation and runtime boundary

- New `crates/dialogue/src/offers.rs`: schema/evidence validation, bounded span
  candidates, learned context matching, unique-value selection, ambiguity refusal,
  and `OfferProposal` provenance (source turn, UTF-8 byte start/end, rule, value).
  Rejects sources over 80 tokens; candidates at most 16 tokens/12 whitespace
  words. Negation/correction/alternative markers reject both source and user.
- `multi.rs`: optional `offers` field defaults absent. After the unchanged tracker
  and reference resolver, fills only missing noncategorical input slots of
  predicted-active services. One SYSTEM source is held; a newer SYSTEM replaces
  it and the next USER consumes it, even when no proposal is accepted. Fresh
  session reset clears it. No old offered value is retried from long history.
- `lib.rs` exports offers. Crate-local `dialogue-multi.rs` adds proposal metadata
  only when nonempty and initializes new training with no optional resolver.
  Legacy/main CLI is untouched; frozen output remains byte-identical.
- New `train.py`, `audit.py`, `evidence.py`, `reproduce.sh` train/assemble the
  additive model, inspect ALL changed claims after inference, verify source and
  frozen artifacts, and reproduce the measured pipeline. No answer store or LLM.

Inference receives ONLY schema/model plus speaker/utterance arrays and its own
predicted state. `InputTurn` rejects unknown annotation fields. All selected
service frames are materialized for every USER turn, including inactive stored
frames. Gold annotations are opened by evaluation/audit only after prediction.

Final review placed the optional selector after the frozen reference component
so its fills are preserved, and replaced unchecked vector indexing for clippy.
Final rebuilt sample then full predictions are byte-identical to the initially
scored H2 candidate. No model/settings changed after dev evidence.

## Unchanged-denominator measurements

| Scope | Joint turns before→after | Exact slot frames before→after |
|---|---:|---:|
| First 10 complete conversations | 31→35 / 91 | 44→48 / 108 |
| Full 179 conversations | 595→609 / 1119 | 689→712 / 1218 |
| Multi-service 54 conversations | 138→152 / 492 | 221→244 / 591 |
| Single-service 125 conversations | 457→457 / 627 | 468→468 / 627 |

Full fuzzy joint sum **761.7596→784.7596 / 1218**, using unchanged upstream
metric. All **3357 predicted frames** remain present. Active F1 remains
**93.2005197055%**, exact routing **977/1119**, correct switches **42/184**.
Multi exact Weather **130/168** and Travel **91/158** are unchanged;
RideSharing becomes **23/265**. Frozen legacy **202/257 exact**, **256/257
intents**, fuzzy **229.18/257 = 89.17509727626459%**, byte-identical predictions.

Full joint transitions: **595 right→right, 14 wrong→right, 460 wrong→wrong,
50 refused→refused**. Slot-frame transitions: **689 right→right,
23 wrong→right, 506 wrong→wrong**. No right→wrong/refused transitions.
`reports/transitions.json` retains every changed joint ID. `reports/first.json`
and `full.json` are unchanged evaluate.py outputs. Baseline routing report and
previous rejected routing conclusions remain unchanged.

## Every added wrong claim and inherited errors

`reports/additions.json` records all **11 initial selection events** with the
verbatim source utterance, user utterance, rule index and exact byte provenance,
and all **32 changed stored slot-turn claims**. All 32 are scored and correct;
**0 new wrong claims, 0 changed unscored held claims, 0 new wrong events**.
The 32 slot additions produce 23 exact frames; other slots/intent/routing still
prevent full correctness on some turns. All 3325 other stored frames are identical.

Inherited Travel.location errors remain explicitly recorded and verified in
`reports/inherited-errors.json`:

| Conversation | USER turn indices | Wrong inherited value | Gold |
|---|---|---|---|
| 17_00022 | 6, 8, 10 | San Rafael | Philly |
| 17_00025 | 6 | Washington D.C | Washington D.C. |
| 17_00034 | 6, 8, 10 | Washington D.C | Washington D.C. |
| 17_00038 | 6, 8, 10, 12 | San | San Fran |

Every Weather/Travel frame is byte-structurally identical to the accepted
reference baseline. This batch neither fixes nor excuses these known errors.

## Validation and reproducibility

- Six focused library tests pass: the four inherited tests plus two new tests
  for verbatim Unicode byte provenance, schema/refusal, conflicting/multiple
  candidates, negation, source consumption/replacement, filled slots and reset.
- Crate fmt check, all-target clippy with `-D warnings`, release build, Python
  AST/Bash syntax and `git diff --check` pass. Initial clippy indexing complaints
  were fixed without suppression; final full prediction bytes are unchanged.
- Full 179-conversation reverse-order replay matches; annotation-bearing input
  is rejected. Frozen model without offers reproduces all previous prediction
  bytes. Legacy prediction bytes match the earlier frozen result.
- `evidence.py` checks every one of the 150 allowed raw source/license files by
  size/SHA256 against the unchanged source manifest, structural frozen-weight
  equality, unchanged routing, and all inherited incorrect claims.

All cargo/training/inference commands used TMPDIR, `systemd-run --user --scope
-pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`;
cargo used `-j1` except fmt; tests `--test-threads 4`. Outer scope independently
verified 8589934592 bytes and zero swap. No workers, LLM calls, credentials, API
keys, services, downloads, held-out/test data, test generators, old private logs,
or commits used. No jobs remain running. Host Astra/fast/login settings cannot
be independently attested by this already-running worker; no new model call or
alternative authentication was launched.

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/offers-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/offers-run/target
export SGD_ROOT=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full
bash tools/dialogue/offers/reproduce.sh
```

Worker namespace uses `SGD_ROOT=/home/gsc/.cache/glassmind/data/sgd-full`.
The script reuses the existing metrics interpreter, source and frozen models;
no download/preparation/evaluation definitions change. Its commands were run
individually during this batch; no claim of a redundant entire-script rerun.

## Source, bytes, hashes and limits

No new external data: **0 downloaded bytes**. Official SGD source
`google-research-datasets/dstc8-schema-guided-dialogue`, revision
**e852981ae34990f4358979625854259302feaa78**, **CC-BY-SA-4.0**; 150 allowed
train/dev/schema/license files, **504306737 bytes**. Exact raw file hashes plus
selected inputs, evaluator, binary and model hashes are in `reports/artifacts.json`.
Training-derived weights and data-bearing reports retain CC-BY-SA-4.0.
Upstream scorer remains Apache-2.0, revision
`5b09c22d73a9d35eb6c5d2a99b95677a45053466`, 11209 bytes, unchanged.

| Artifact | Bytes | SHA256 |
|---|---:|---|
| Selected train | 45291003 | `f347a5708e4444998ddcb7546ab5ecab68cac1c7785a12db18d7ad12920619f3` |
| Selected dev | 1207178 | `89cc1943a0310354be053988235ff64e0cc232fd57a86b82b5f0f1dc354f8bf7` |
| Train schema | 3707 | `7afda6fb434a47ea594edb4ebdb9b182194156df4aaf1c5e40f984cc624eaef4` |
| Dev schema | 3707 | `b021ae923dbeb7782b99ac6628e65b1b48f3cca2892fb62ce8c77b472324e089` |
| Frozen references model | 32169632 | `9991b341b4da7e775e0fa7e3ca55a2a00f4935a0a7b94db16f02b2b6596be83c` |
| Candidate model | 32174437 | `deddfaac2f596c5090aeb65bf77d24470d1d44f61d2aad69469792cd26232781` |
| Offers weights | 8132 | `e2ef37d30e47ac137c2b8afaa429450ef6b69ccfc1b49461809951eb125d446a` |
| Full predictions | 404892 | `edc590e7281c72bf09890c274a87cc51abce174e124ca84ba17754efa4106ba6` |

Large artifacts persist only under `offers-run/tmp/offers/`; binaries use the
existing `offers-run/target/`. Source data remain in the existing read-only mount
and persistent host path above. Small code/reports remain in the owned checkout.

Limits: all 11 measured selections occur at later system confirmations. Initial
cross-service "there" requests usually remain unresolved until then. This is
restricted three-service dev evidence, not general SGD or held-out performance.
Proper-name shape loses lowercase names and unusual punctuation; sparse empirical
calibration does not guarantee out-of-domain correctness. Negation/ambiguity
refusal is conservative lexical/candidate based, not general semantic negation
or correction understanding. It cannot fix existing wrong filled slots, disambiguate
an unrecognized alternative, or recover stale sources beyond one SYSTEM turn.
No broad attraction discovery, unseen-service transfer, execution, or source
correction capability is claimed.
