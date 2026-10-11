# Span sequence handoff — REJECTED at TRAIN calibration

**Do not integrate the runtime patch or weights.** One bounded structured-sequence
experiment completed its fit/calibration stage and failed its preregistered gate.
No candidate threshold was selected, no candidate DEV inference occurred, and
there is **no measured capability improvement**. All `crates/dialogue` edits are
restored byte-for-byte to base **69ba975a9f44bd8f3aaf45be355fffe82cbc6577**.
Only new `tools/dialogue/spansequence/` remains uncommitted. No commits.

The requested candidate first10/full179 acceptance evaluation was **not reached**:
it would require inventing a fallback threshold after the CAL gate failed.
The first10/full179 runs below are explicitly **optional-feature-absent regression
replays**, not results of the rejected sequence model. No lowered gate, second
representation, dev lexicon, contiguity patch, transfer suppression or scorer edit.
Root integration owns any next experiment.

## Parent review

Original22-file/1,605,593-byte submission archived and hash-verified. Parent
verified50 distinct artifact paths, including exact candidate source reconstructed
from the unapplied patch, and independently recomputed all six CAL thresholds
across2505 rows. All fail the original99%/minimum30 gate. Runtime restored,
worktree removed and temporary login link unlinked. No candidate inference or
training repeated by parent. [Parent evidence](reports/parent-review.json).

## Hypothesis and one fixed experiment

[EXPERIMENT.md](EXPERIMENT.md) was written after bounded TRAIN-only inspection and
before implementation/DEV. Hypothesis: joint BIO emissions and learned transitions
with reusable token/context features can recover missing correction/referent spans
without losing any610 baseline correct joint turns or adding wrong assertions.
Acceptance was >=610joint, >=716frames, all610 retained, zero new wrong changed
assertions and a strict substantive joint/frame gain or inherited-error repair.

One per-service averaged structured perceptron, legal BIO transitions, eight epochs
in deterministic original FIT order, unit updates and post-sequence averaging.
Features: lowercase word/shape/capital/numeric windows +-2, current prefix/suffix
1..3, first/last, up to32 distinct previous-system words. FIT-only vocabulary
frequency>=2, max50000/service; max256 tokens,4096bytes/input,32labels.
Weights have13914 feature rows across three services (3918/5796/4200), no embeddings,
pretrained model, answer lookup table or annotation-bearing runtime type.

TRAIN split is SHA256(dialogue_id) integer mod5==0: **2532FIT/642CAL**, out of
3174 selected conversations/34022USER turns. Four noncategorical input slots are
selected from full TRAIN schema: destination/location/city/date. FIT has10257
selected-service sequences; CAL2505 with552 gold spans. The old frozen models
already used all TRAIN, including CAL; **only new weights hold CAL out of fitting**.
No whole-TRAIN refit. The minimal Rust trainer consumes an offline supervision
file, separate from the runtime's speaker/utterance-only API.

Confidence is the minimum exact token max-marginal gap across a proposed BIO span
and its neighboring boundary tokens. Select the lowest of [0,2,4,8,16,32] with
at least30 raw CAL proposals and99% exact annotated-span precision. This is
empirical span precision, not a probability or downstream joint correctness.

| Threshold | Correct / proposed CAL spans | Exact precision |
|---:|---:|---:|
|0|512/535|95.7009%|
|2|510/533|95.6848%|
|4|507/528|96.0227%|
|8|499/517|96.5184%|
|16|490/505|97.0297%|
|32|458/470|97.4468%|

**All six fail.** Threshold32 also recovers only458/552 annotated spans (82.9710%).
[train-report.json](reports/train-report.json) preserves all2505 CAL rows, raw
proposals, gold span endpoints and training epoch mistakes. No threshold above32
was tried. [split.json](reports/split.json) records exact permitted TRAIN IDs.

The initial trainer exited on CAL rejection before saving weights. It was amended
only to preserve rejected weights and detailed CAL rows, then the identical
schedule was rerun. The observed epoch mistakes and threshold totals matched the
first run; an independent first-run weights hash is unavailable. No representation,
update, training order, threshold or acceptance criterion changed. The saved
[unselected-weights.json](unselected-weights.json) deliberately has **no threshold**
and cannot deserialize as a runtime SequenceModel.

## Concrete causal evidence and limits of that evidence

[cal-audit.json](reports/cal-audit.json) enumerates all23 raw span errors and all12
remaining at32. Four high-margin boundary errors: two destination names extend
past a period into the next sentence, one address loses its building suffix, one
multiword city loses two words. Excluding separator punctuation from emissions
leaves the chain able to continue into capitalized sentence starts; transitions
do not by themselves solve span completeness or sentence boundaries.

Eight high-margin proposals have no corresponding span annotation. Several are
hotel/flight dates uttered after Weather discussion; this illustrates that a
recognizable date is not necessarily a correction to the Weather referent.
A separate diagnostic comparison against CAL gold state finds four of those eight
already equal a stored gold value. **Do not call all12 downstream wrong assertions.**
Eight of the12 differ from current exact gold values; four match. This post hoc
diagnostic did not change calibration labels or the gate. Source occurrence alone
is not evidence of intended referent. Routing and downstream state application of
the rejected model were not evaluated; raw proposal errors cannot establish which
would become newly wrong claims versus already wrong or inactive states.

## Exact rejected implementation and validation

[rejected.patch](rejected.patch) contains only owned crate edits:

- `spansequence.rs`: bounded tokenizer with byte/character offsets, reusable
  features, learned transitions, constrained Viterbi and exact max marginals,
  per-slot verbatim proposals and model validation.
- Offline `dialogue-spansequence-train` and utterance-only diagnostic probe.
- Optional MultiModel v3 field; sequence spans overwrite/add after unchanged
  service advance and before unchanged references/offers. One longest qualifying
  span per slot wins; no deletion. Past predicted states are still snapshotted
  before all service updates. Per-turn provenance clears deterministically.
- Crate-local `dialogue-multi` snapshot field skips serialization when empty;
  lib export and existing test model constructors initialize the optional field.
  Legacy tracker, root main CLI, routers/intents/categorical/reference/offer/
  boundary weights, shared manifests and scorers are untouched.

Final **12 focused Rust tests pass**, including four new tests: exhaustive tiny
path enumeration checks Viterbi/max-marginal arithmetic; illegal initial I and
input bounds; authored unseen-name/multiword/punctuation/correction/Unicode
positives and negative; optional correction, persistence, reset, version and NaN
validation. These use transparent toy weights, not claims of trained-model
accuracy on new names. An initial toy fixture overextended lowercase words; only
its toy weights were corrected before final passing tests. Learned weights and
schedule were not changed by that fixture fix. All-target clippy `-D warnings`,
release build, fmt, Python AST, shell syntax and diff whitespace checks pass.

Hashes of model/source/binaries were frozen before any DEV replay in
[freeze.json](reports/freeze.json). Source aggregate SHA256:
`8361d7d30053e8bffc95aec5a1ebaf0a0361b79987b5c358fdc0ea9a2cdb02e8`.
[validation.json](reports/validation.json) records the exact checks and limitations.

## Optional-absent regression evidence, not candidate results

Unchanged evaluate.py/compare.py and existing metrics venv were used, first10 then
full179, all1119USER turns and1218gold frames. First10 is35/91joint and48/108frames;
full is610right/459wrong/50refused and716/1218frames. All610correct retained and
all3357 stored predictions retained. Full54multi-service dialogues remain153/492
joint,248/591frames. All predictions match frozen v2 **byte-for-byte**. No coverage
or refusal changes. There are **zero changed slot assertions**, because this replay
has the optional model absent; this is not a sequence-model acceptance result.

All seven inherited wrong slot-turns remain explicitly joined in
[audit.json](reports/audit.json): 17_00022 turns6/8/10 and17_00038 turns6/8/10/12.
All four previously repaired scored boundary assertions stay correct. No new held
states are credited; unchanged held states have no extra accuracy credit.
[transitions.json](reports/transitions.json) reports610right→right,
459wrong→wrong,50refused→refused,716correct-frame→correct-frame and502wrong→wrong.

Full179 reversed conversation order is identical; annotation-bearing input rejects.
Legacy45 predictions match prior bytes:202/257strict,256/257intents,
**89.17509727626459%fuzzy**. [legacy.json](reports/legacy.json) retains the metric.
[artifacts.json](reports/artifacts.json) verifies every restored runtime source and
unchanged scorer, plus exact sizes/hashes of evaluated binaries and predictions.

| Artifact | Bytes | SHA256 |
|---|---:|---|
|Frozen approved v2 model|32174720|`fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88`|
|Unselected sequence weights|652222|`70aadf00846b9434fcafdf958a3b83b2d1f62dfa1599562f14337b520cb9dafe`|
|Full v2 replay predictions|405228|`8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d`|
|First10 v2 replay predictions|34694|`91afdda73ca8e79afcc1a51d204fc15d1c8fb26efbfbec5b4e69b9cbeafd6761`|
|Legacy predictions|49012|`3b9a21719f738bf40feb8dfc86b266be5fe07ff48c49ea8ebe4b5ebfa79623b5`|
|Rejected source patch|36088|`dcf3a84154fe218b639a32e36949f559667ade0808d980fb798de897b87f7045`|

## Reproduction, provenance and handoff state

In a clean isolated checkout of the base above with this evidence directory:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/spansequence-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/spansequence-run/target
bash tools/dialogue/spansequence/reproduce.sh
```

The script temporarily applies rejected.patch and reverses it on exit. It expects
trainer exit1 with exactly the recorded rejected CAL totals; it never substitutes
a threshold. It then runs optional-absent first/full/legacy evaluation. Individual
commands and evaluate.sh completed in this worker; the consolidated reproduce.sh
was syntax checked, **not rerun end-to-end**. It uses the retained local paths;
for separate scratch, preserve the baseline at the same RUN/baseline layout used
by freeze.py and the explicit paths in evaluate.sh. Full artifacts/executables are
at `/home/gsc/.cache/glassmind/continuous-20261010/spansequence-run/tmp/spansequence`. The target directory contains experimental executables; no candidate
runtime model exists. Rebuild restored source before using new production binaries.

**External data downloaded:0bytes.** Reused public SGD revision
`e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0, selected TRAIN45291003bytes,
DEV1207178bytes, full TRAIN schema75764bytes, DEV schema3707bytes, license20139bytes.
Exact consumed-file hashes are in freeze.json. The derived supervision file is
2474329bytes; additional unselected model JSON652222bytes (<40MiB). Data-bearing
reports/weights retain CC-BY-SA-4.0. No new dependencies. Upstream metric remains
Apache-2.0, pinned revision5b09c22d73a9d35eb6c5d2a99b95677a45053466,11209bytes.

Every cargo invocation set on-disk TMPDIR, reused CARGO_TARGET_DIR and ran under
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, -j1 except fmt, tests--test-threads4. Training and inference
used the same bounded scopes and were monitored to completion. Outer scope
verified8589934592bytes/0swap. No owned jobs remain. No agents, LLM calls, services,
credentials/API keys, downloads, /tmp, held-out/test/hidden content, private logs,
commits or out-of-ownership edits. This worker cannot independently attest the
host's model/login configuration and made no alternate model/authentication call.

This rejection establishes a limitation of one sequence representation and
confidence schedule on the allowed TRAIN/CAL distribution. It does not measure
candidate DEV joint accuracy, prove semantic referent freshness, or support a
broader SGD/held-out accuracy claim. Parent independent retraining/replay is still
required before any future publishing decision.
