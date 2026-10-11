# Bounded multi-service dialogue batch — experimental, not promoted

Base source: `eb15517c792fabed81e82b0733394e354b96e195`; uncommitted changes only.
Ownership: `crates/dialogue/` and new `tools/dialogue/`. No shared scorer,
manifest, main CLI, README, historical prompt, or other track changed. No agent,
LLM call, credentials, API key, held-out item, test split, hidden-test code,
test generator, or old private log was used. This worker cannot independently
attest the already-running parent model configuration; no new model calls were
launched. New statistical training used official training annotations only.

**Outcome:** usable utterance-only multi-service library and standalone CLI,
with a measurable routing/switching gain. Neither experimental model satisfies
all preregistered end-to-end acceptance criteria. Do not promote either as a
successful general multi-service state tracker. The frozen legacy result is
preserved exactly. Work on cross-service state/reference resolution is incomplete.

## Hypotheses and selection, fixed before scoring

1. Select RideSharing_1 plus Travel_1 from train/dev schema availability and
   training co-occurrence counts: Travel co-occurs in 171/688 RideSharing train
   conversations, more than any other service available in both schemas.
   Selection: every complete dev conversation with nonempty service set contained
   in the selected set, across **all 20 dev shards**. Structural scan produced
   90 single-service conversations and zero multi-service conversations. Reject
   this scope before evaluating new dev labels: it cannot establish the capability.
2. Add Weather_1 before new dev-label scoring. It appears in both schemas, has
   951 training conversations, and co-occurs with RideSharing in 83 train
   conversations. Travel has 1,871 train conversations. Freeze three services,
   same selection rule, and first 10 lexicographic conversation IDs; also first
   10 multi-service IDs. They happen to be the same sample after the expansion:
   `17_00004` through `17_00013`, **91 user turns / 108 frames**.
   Hypothesis: 12-epoch learned routing beats 0-epoch train-centroid routing in
   active-service F1 and exact full-joint turns on the multi sample and full
   selection, with no first-sample regression, correctly predicting switches;
   preserve frozen RideSharing 202/257 exact and 89.18% fuzzy. **Rejected:** sample
   F1 regresses and exact states do not improve.
3. After that failure, fix one follow-up before editing/scoring: independent
   binary service classifiers should learn simultaneous-service activity better
   than a categorical classifier over service combinations. Same 12 epochs,
   feature settings, samples and full denominator. Require better F1 **and**
   exact full-joint turns than candidate 1 on sample and full selection.
   **Rejected as an end-to-end model:** routing improves substantially, sample
   joint states tie, and full joint states lose one. Retain the implementation
   and evidence as an explicit experiment; do not silently relax acceptance.

Training selection: all conversations containing at least one selected service,
**3,174 of 16,142 train conversations**, 34,022 user turns, 12,762 selected-service
frames (RideSharing 3,455; Travel 6,491; Weather 2,816). Other-service turns provide
negative router supervision, never other-service slot labels. Tracker training
uses gold previous train states; inference uses its own predictions.

Dev selection: **179 of 2,482 complete conversations**, **125 single / 54 multi**,
**1,119 user turns / 1,218 annotated service frames**. Single: 627 turns/frames;
multi: 492 turns / 591 frames. The other 2,303 conversations are excluded solely
because they contain an unselected service. No turns/frames inside a selected
conversation are ignored. Every ID, source shard hash, sample ID, and count is
in `reports/data-manifest.json`. This is a scoped dev result, not whole-SGD or
independent held-out performance.

## Exact changes

- `src/multi.rs`: trained service-set router and optional independent binary
  routers; `MultiSession` has separate persistent predicted state per service.
  System text updates context; user text plus previous predicted activity selects
  services to advance. Inactive state remains available. Multi-service output is
  allowed. All predictions are schema filtered. No service-name heuristics,
  runtime lookup table, service execution, or runtime LLM.
- `src/bin/dialogue-multi.rs`: standalone `train` and `predict`. Prediction input
  is an array of independent conversations, each containing only `speaker` and
  `utterance`. Every selected service frame is output for every user turn,
  together with the predicted active-service set. Annotation-bearing input is
  rejected. Frozen per-service models can be supplied through repeated `--frozen`.
- `src/tracker.rs`: generic `ProposalModel` supports learned models or an
  empirical single-label model with persisted evidence count. Empty evidence is
  rejected; no fabricated second class. Existing learned v2 JSON remains readable.
  This handles sparse future services; these selected services did not require
  a constant slot fallback. Existing span/intent/state semantics are unchanged.
- Crate-local `Cargo.toml`: `default-run` preserves existing package command
  behavior despite adding a binary. Shared manifests and old CLI untouched.
- New scripts cache explicitly permitted pinned shards, strip runtime input,
  call unchanged upstream metrics, report all correctness transitions, and
  reproduce the batch. Two focused library tests cover annotation rejection,
  empty/one-class evidence, schema filtering, switching, independent state,
  simultaneous services, and fresh session state.

## Measurements

All service trackers have identical weights across router variants: frozen
RideSharing plus Travel/Weather trained 12 epochs. Router features: word/character
TF-IDF, char_weight=0.15, min_df=2; deterministic shared averaged-margin trainer,
0 or 12 epochs. No threshold tuning. Independent variant has three binary models.

“Active” here means presence of a service frame on that USER turn, including
frames whose active_intent is NONE. Precision/recall score these sets. Strict
frame score and fuzzy score call the existing pinned upstream SGD goal metric
with fuzzy matching disabled/enabled respectively. Full-joint turn requires
exact service set, exact slots for **every** gold frame, and correct intents.
All three predicted frames are materialized before looking at gold; extra active
services count as false positives and make the full-joint turn incorrect.
Requested-slot metrics remain outside the existing tracker/scorer scope.

| Scope / router | Active P / R / F1 | Exact slot frames | Full-joint turns | Fuzzy joint sum | Correct switches |
|---|---|---:|---:|---:|---:|
| Sample centroid | 86.79% F1 | 28/108 | 21/91 | 29.17/108 | 6/33 |
| Sample joint 12 | 86.43% F1 | 28/108 | 21/91 | 29.17/108 | 8/33 |
| Sample independent 12 | 96.12 / 91.67 / 93.84% | 28/108 | 21/91 | 29.17/108 | 18/33 |
| Full centroid | 89.67 / 90.48 / 90.07% | 596/1218 | 501/1119 | 663.6196/1218 | 38/184 |
| Full joint 12 | 98.63 / 88.34 / 93.20% | 598/1218 | 549/1119 | 664.1996/1218 | 42/184 |
| Full independent 12 | 99.29 / 92.20 / 95.62% | 595/1218 | 548/1119 | 661.1996/1218 | 116/184 |
| Multi only centroid | 89.03 / 85.11 / 87.02% | 128/591 | 89/492 | 146.7296/591 | 38/184 |
| Multi only joint 12 | 96.92 / 79.86 / 87.57% | 130/591 | 92/492 | 147.3096/591 | 42/184 |
| Multi only independent 12 | 98.48 / 87.65 / 92.75% | 127/591 | 90/492 | 144.3096/591 | 116/184 |

Switch correctness requires both previous and current predicted activity sets
match gold at a gold set change. This includes adding/dropping simultaneous
frames, not only switching between disjoint services. First turns are excluded.
Full predicted-frame denominator is always **3,357**, not just the 1,218 gold
frames. See reports for per-service and single-service breakdowns.

Regressions are material, not hidden:

- Centroid→joint12: full-joint right 501→549; 497 stay right, 47 wrong→right,
  5 refused→right, **2 right→wrong and 2 right→refused**. Wrong/refused totals:
  602/16→520/50. Slot-frame transitions: 4 right→wrong, 6 wrong→right.
- Joint12→independent12: full-joint right 549→548; 545 stay right,
  1 wrong→right, 2 refused→right, **2 right→wrong and 2 right→refused**.
  Wrong/refused totals: 520/50→511/60. Slot-frame transitions: 6 right→wrong,
  3 wrong→right. “Refused” means no active service predicted, while all three
  stored frames still appear in output. These cases remain in every denominator.
- Frozen legacy command: **45 conversations, 257 frames; 202 exact slot states,
  256 correct intents; fuzzy sum 229.18, mean 0.8917509727626459**. Final binary
  output is byte-identical to the earlier baseline reproduction. The embedded
  RideSharing model is structurally identical to the frozen JSON. Its model
  file has not been changed.

## Validation and isolation

Final targeted release build, fmt check, all-target clippy (`-D warnings`), and
2/2 focused unit tests pass. Bash syntax and Python AST checks pass. Fixed
sample preceded full evaluation for each variant. Final full independent replay
matches its first full predictions byte-for-byte. Full reverse-conversation-order
replay is identical after reversing results (179 fresh sessions); injected
`frames` is rejected. `reports/final-audit.json` records the audit. Runtime process
receives no dialogue IDs, services, frames, spans, actions, API results or labels.
User/system utterance text is permitted context. The adapter consults annotations
only after the subprocess emits all predictions. Separate unit tests exercise
state carry on switches and simultaneous service predictions.

No workspace-wide tests were run: only this owned crate changed. The reproduction
script assembles the individually executed commands; no claim that a second
complete download/training run was necessary or performed. A first training
attempt targeting the read-only frozen model directory was stopped through its
exact systemd scope and rerun under this worker's writable artifact directory.
All owned jobs are stopped/completed at handoff. Outer scope and every cargo,
training and inference job use 8 GiB MemoryMax, zero swap, separate process group,
600-second timeout, 10-second kill grace. Cargo uses -j1 and tests four threads.

## Sources, licenses, bytes and retained artifacts

Dataset: official `google-research-datasets/dstc8-schema-guided-dialogue`, revision
`e852981ae34990f4358979625854259302feaa78`, **CC-BY-SA-4.0**. Explicit pinned raw
URLs under `/train/` (127 shards), `/dev/` (20 shards), two schemas, and LICENSE.txt
only. Total **504,306,737 bytes**, below 600,000,000; at most four requests. Full
permitted shards cached outside git at `/home/gsc/.cache/glassmind/data/sgd-full/`
to avoid redownloading. Every cached byte is verified against the previous source
manifest. No archive, repository tree, or test URL used. Derived selections live
in its `selected/` directory. Source license retained in the cache.

Metric: unchanged `crates/dialogue/tools/upstream_metrics.py`, Google Research
revision `5b09c22d73a9d35eb6c5d2a99b95677a45053466`, **Apache-2.0**, **11,209 bytes**,
SHA256 `709667d982b99d1bf1f2be48fc47418b7b7fc9a5f1c43e9fe8dd987828a51df5`.
NumPy 2.3.3, fuzzywuzzy 0.18.0; pure-Python difflib backend (no Levenshtein).

Model/artifact root:
`/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/multi/`.

| Artifact | Bytes | SHA256 |
|---|---:|---|
| model-0.json | 31,583,903 | `18303a59e90591b318961ffd00e98daa583923aed76e3ef93f0eb6a7af508ada` |
| model-12.json | 32,200,901 | `7ad9f82b9adfd5671511feeca787193ce82ea2bf5f781ea5c6353befdff3428f` |
| model-independent-12.json | 47,605,514 | `f76a8996c8bac8aa23b8d82ff2f3c225c6539e75fafe1d6a355b483d14fd1af6` |
| Frozen RideSharing model | 7,583,366 | `f3759d541be8115043b68ade07ff80856acfd9e8e403e3071e1fd4a1eec2b547` |

`reports/artifacts.json` records exact absolute paths, binary/source hashes,
derived dataset bytes/hashes and prediction hashes. Raw predictions, stripped
inputs, reversed inputs and local new training output remain under that root.
Model weights derive solely from the CC-BY-SA training source; no new external
answer store or lookup data was introduced.

## Reproduce and use

From the worktree root (the existing target directory is required):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/target
bash tools/dialogue/reproduce.sh
```

All exact commands and caps are in the script; it verifies/reuses the full source
cache, builds/tests, trains fixed models, evaluates fixed samples then full dev,
audits isolation, scores frozen RideSharing and reports correctness transitions.
It writes artifacts outside git. The already-frozen RideSharing model is required.

For real use, save `[[{"speaker":"USER","utterance":"Find somewhere to visit"}]]`
to a file, one inner array per independent conversation, then:

```bash
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  "$CARGO_TARGET_DIR/release/dialogue-multi" predict \
  --input "$TMPDIR/conversations.json" \
  --schema /home/gsc/.cache/glassmind/data/sgd-full/selected/dev-schema.json \
  --model "$TMPDIR/multi/model-independent-12.json" \
  --output "$TMPDIR/predictions.json"
```

Limitations: cross-service references and system-offered entities are not resolved;
no date/number normalization was added. In the multi-service diagnostic sample,
all exact successes are Weather frames; Travel and RideSharing have zero exact
frames. That is why stronger routing does not improve its joint state score.
No unseen-service generalization, calibration, reliable out-of-domain refusal,
requested-slot prediction, service execution or broad conversational competence
is claimed. Full model promotion remains rejected; coordinator may integrate
useful infrastructure with this limitation or continue a separate preregistered
reference-resolution experiment. No commits were made.


## Coordinator continuation

This is the retained rejected routing experiment. The later grounded-slot
transfer experiment keeps its joint12 weights fixed and is reported separately
in [references/HANDOFF.md](references/HANDOFF.md). The full source cache was
preserved at `~/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full`;
preparation/reproduction now accept `SGD_ROOT` and default to that persistent
location. This path correction does not change selection or scoring.
