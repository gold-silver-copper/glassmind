# Boundary restoration handoff — accepted bounded candidate

Uncommitted changes based on `2621cf42b33d32b228f3decbe68988917eec6991`, restricted
to `crates/dialogue/` and new `tools/dialogue/boundaries/`. No main CLI, shared
manifest/README, scorer, loop, other track, historical prompt or lockfile edits.
No commits. Coordinator must integrate these uncommitted files and retained model.

**Acceptance met:** full selected dev improves **609→610/1119 exact joint turns**
and **712→716/1218 exact frames**, retaining every609 correct joint turn and
all712 correct frames. Final610right/459wrong/50refused. Two inherited punctuation
copy errors are corrected across four scored slot-turns; **zero new wrong claims**.
Six changed held slot-turns have no gold frame and are explicitly **unscored**.
The stale-source and truncated-name errors remain: **two events, seven scored
slot-turns**. This is a narrow exact-boundary gain, not a semantic freshness fix.

## Preregistered hypothesis and rejected training variant

[EXPERIMENT.md](EXPERIMENT.md) records the hypothesis/acceptance before runtime
edits and the TRAIN-only schema amendment before any dev evaluation. No dev
strings/values were added to runtime or used to fit/select boundary rules.

The frozen tokenizer excludes final punctuation. TRAIN annotations distinguish
punctuation that belongs to a value from ordinary sentence endings. Learn the
endpoint using final-word shape (alphabetic/numeric runs anonymized as A/D plus
length capped at3), following punctuation, and a suffix length. No entity values,
service IDs or slot names are stored in boundary weights.

Initial USER-only inspection found no calibration support for the abbreviation
boundary. H1 using selected-service noncategorical input slots also rejected it,
and retained only a parenthesis rule. That training variant was rejected before
dev and its full evidence retained. Unconditional punctuation inclusion was
rejected by abundant TRAIN negatives. BIO contiguity and semantic correction
freshness were deferred as separate hypotheses; no decoder changes were made.

H1b uses **the same3174 selected TRAIN conversations**, including their existing
annotations for other services, and the public full TRAIN schema to identify all
noncategorical input slots. The selected JSON already contains these annotations;
no new conversations or datasets were added. Both USER and SYSTEM spans contribute.
This scope amendment is disclosed because selected-service-only evidence would
not support the accepted abbreviation rule.

Split fixed as SHA256(dialogue_id), interpreted as an integer, mod5==0:
2532fit/642calibration;34022USER turns. Keys/output lengths proposed on fit only.
Thresholds fixed before dev: at least5 distinct fit and3 calibration conversations,
zero mismatched endpoints across both splits. One candidate was evaluated on dev.

| Accepted shape / following punctuation | Extension | Fit/cal conversations | Fit/cal rows | Distinct annotated values |
|---|---|---:|---:|---:|
| `A1.A1` / `.` | first1 character |47/7|73/11|18|
| `A3` / `).` | first1 character |12/6|14/6|4|

The abbreviation evidence is12USER/72SYSTEM spans; parenthesis18SYSTEM/2USER.
Distinct annotated strings do not mean distinct cities or independent entity
families. Calibration is empirical gold-span endpoint evidence, **not calibrated
probability or a frozen-predicted-source TRAIN evaluation**. No classifier retraining.

## Exact code and runtime boundary

- New `crates/dialogue/src/boundaries.rs`: validates bounded evidence-bearing
  rules; locates the most recent exact, case-sensitive, whole-value occurrence
  in prior utterance history; applies only a matching learned punctuation rule.
  Returns source turn, UTF-8 byte start/end, original/corrected value and rule.
  Every correction is a contiguous verbatim source slice. It never synthesizes
  punctuation, scans backward past a newer matching occurrence without evidence,
  or suppresses an existing transfer. Conflicting corrections decline to change it.
- `multi.rs`: optional boundary model is applied after the unchanged reference
  proposal, only when filling a missing slot. Source predicted states, classifiers,
  routing and offers stay frozen. Boundary proposals are cleared each turn;
  session reset discards history. Same-turn source predictions cannot leak in.
- New optional mechanism requires **MultiModel version2**; submodel version1.
  Version1 frozen models with absent boundaries retain behavior. A v1 model with
  boundaries rejects, and old binaries reject v2 rather than silently ignoring it.
- Crate-local `dialogue-multi` emits boundary provenance only when nonempty.
  `lib.rs` exports the module; offers' unit-test struct initializes the new field.
  Legacy tracker/main CLI is unchanged.
- New train/audit/evidence/reproduction tools and small reports live here.
  Gold annotations enter training or post-inference audit only. Runtime still
  deserializes only speaker/utterance; schemas/models/prior predictions are allowed.
  IDs, gold frames, actions and slot annotations cannot enter inference.

## Exact measurements and complete changed-claim audit

| Scope | Joint turns baseline→candidate | Exact frames baseline→candidate |
|---|---:|---:|
| Fixed first10 complete conversations |35→35/91|48→48/108|
| Full179 conversations |609→610/1119|712→716/1218|
| Multi-service54 conversations |152→153/492|244→248/591|
| Single-service125 conversations |457→457/627|468→468/627|

Multi-service Travel91→95/158; Weather130/168 and Ride23/265 unchanged.
All3357 stored predicted frames remain. Routing977/1119 and F1 93.2005197055%
unchanged; all active intents and offer proposals unchanged. Full fuzzy joint sum
784.7596/1218 is unchanged because fuzzy normalization already ignores punctuation.
Joint transitions609right→right,1wrong→right,459wrong→wrong,50refused→refused.
Frame transitions712right→right,4wrong→right,502wrong→wrong.

[reports/audit.json](reports/audit.json) includes every one of ten changed slot-turns,
both initial correction events, exact utterance/byte provenance, all gold values,
all six unscored held changes, and an explicit empty new-wrong-claims list.
[reports/transitions.json](reports/transitions.json) uses the unchanged comparator.

All four inherited events are explicitly tracked:

| Conversation | Scored USER indices | Inherited claim | Candidate outcome |
|---|---|---|---|
|17_00022|6,8,10|San Rafael vs Philly|unchanged wrong,3slot-turns|
|17_00025|6|Washington D.C vs Washington D.C.|corrected,1slot-turn|
|17_00034|6,8,10|Washington D.C vs Washington D.C.|corrected,3slot-turns|
|17_00038|6,8,10,12|San vs San Fran|unchanged wrong,4slot-turns|

These values appear only in post-inference reports, never implementation/model
rules. Weather's original truncated source state remains unchanged; correcting
its downstream copy does not claim to repair the source tracker.

## Validation and reproduction

Eight focused Rust unit tests pass, including new Unicode byte provenance,
latest-occurrence handling, evidence/ambiguity rejection, optional-version
validation, source-state preservation, per-turn clearing and reset integration.
Crate fmt check, all-target clippy `-D warnings`, release build, Python AST parsing,
Bash syntax and diff whitespace checks pass. No workspace-wide suite was needed.

Full179-conversation reverse-order replay is identical; annotation-bearing input
is rejected. Frozen baseline with boundaries absent reproduces full predictions
byte-for-byte. Legacy45 conversations reproduce old prediction bytes exactly:
**202/257 exact,256/257 intents,89.17509727626459% fuzzy**. Source hashes verify
all150 allowed raw source/license files and unchanged evaluation files. Structural
comparison proves router/service/reference/offer weights unchanged.

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/boundaries-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/boundaries-run/target
export SGD_ROOT=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full
export BOUNDARY_BASELINE=/home/gsc/.cache/glassmind/continuous-20261010/boundaries-run/baseline
bash tools/dialogue/boundaries/reproduce.sh
```

The namespace used alias `/home/gsc/.cache/glassmind/data/sgd-full` for SGD_ROOT.
Script commands ran in order, with fmt/clippy separately and the remaining
pipeline via `$TMPDIR/boundaries/evaluate.sh`. The latter excludes completed
fmt/clippy; evidence verification ran afterward. Validation output is retained
in `$TMPDIR/boundaries/validation.log`. No claim of a second entire-script replay.
To reproduce rejected selected-schema fitting, run the same train.py command
with `--schema "$SGD_ROOT/selected/train-schema.json"` and a separate output dir.

Every cargo invocation used persistent TMPDIR and the required
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, `-j1` except fmt, and test arguments `--test-threads 4`.
Training/inference/evidence calls were also capped. Outer scope verified8589934592
bytes/0swap. No agents, extra LLM calls, credentials, API keys, services,
background jobs surviving the worker, downloads, test/held-out items or hidden
code, old private logs, or commits. Host model/login cannot be independently
attested by this running worker; no alternative model/authentication was used.

## Source provenance, artifacts and limits

No new external data: **0 downloaded bytes**. Reused SGD from
`google-research-datasets/dstc8-schema-guided-dialogue`, revision
`e852981ae34990f4358979625854259302feaa78`, **CC-BY-SA-4.0**;
150allowed train/dev/schema/license files,504306737bytes. Selected TRAIN45291003
bytes,3174conversations/34022USER turns; selected dev1207178bytes. Full TRAIN
schema75764bytes; selected train/dev schemas3707bytes each. New derived boundary
weights398bytes and data-bearing reports retain CC-BY-SA-4.0. No new dependencies.
Unchanged upstream metric: Apache-2.0, revision
`5b09c22d73a9d35eb6c5d2a99b95677a45053466`,11209bytes.

| Artifact | Bytes | SHA256 |
|---|---:|---|
|Frozen baseline model|32174437|`deddfaac2f596c5090aeb65bf77d24470d1d44f61d2aad69469792cd26232781`|
|Candidate model|32174720|`fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88`|
|Boundary weights|398|`e807be35af0d5264d687279b71db1c6d2f4f48edb5d4238f2b87f6b0ea5a5e01`|
|Baseline full predictions|404892|`edc590e7281c72bf09890c274a87cc51abce174e124ca84ba17754efa4106ba6`|
|Candidate full predictions|405228|`8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d`|
|Evaluated dialogue-multi binary|16829712|`2c5f0e581ad6778d3056b57ee0060fe83dd6a6b1306844f8c330a57523721f1d`|

Large artifacts remain under `boundaries-run/tmp/boundaries/`; baseline under
`boundaries-run/baseline/`; existing target reused. Exact per-file source, model,
data, scorer and binary hashes are in [reports/artifacts.json](reports/artifacts.json).
[reports/source.json](reports/source.json) contains portable relative source names
and an aggregate digest; evaluated source includes the uncommitted code listed.

Limits: two dev events, both from the same dotted abbreviation family; no broad
entity-independent accuracy guarantee. The parenthesis rule has TRAIN evidence
but no observed dev usage. Exact case matching can miss newer differently-cased
mentions. Most-recent occurrence provenance does **not** establish that the
predicted referent is fresh. Existing stale/truncated predictions, BIO O-gap
extension, filled-target corrections and missing words remain unsolved. Six
changed held states are unscored. Restricted three-service dev only; no held-out
or broader SGD claim. No candidate tuning occurred after dev results.
