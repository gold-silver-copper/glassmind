# Dialogue tracking: first SGD batch

This standalone library/binary implements a trained, non-LLM state tracker for
the **complete single-service RideSharing_1 slice of official SGD dev**. This is
not a whole-SGD score, a multi-service routing result, or an unseen-service result.
The slice was selected from schema availability before evaluation. No official
test files were fetched, extracted, read, or scored.

## Implemented

`glassmind-dialogue` has `train`, `eval`, and `predict` subcommands. Its reusable
library stores a separate `SessionState` for each conversation. `InputTurn`
contains only speaker and utterance, and rejects additional fields. The runtime
receives that input, the service schema, trained model weights, and its own prior
predicted state. It cannot receive SGD user frames, slot spans, system actions,
API results, or gold state. New conversations start with empty state.

The small models reuse `glassmind-everyday::IntentModel`: TF-IDF word/character
features, normalized class centroids, then 12 epochs of averaged multiclass
margin updates. Intent and categorical-slot models learn KEEP/SET transitions
(slot models also permit DELETE). Intent proposals see the previous predicted
intent; slot proposals see whether their slot is already filled. A token model
learns BIO spans for noncategorical schema input slots. It extracts values from
the current user utterance, including previously unseen strings. Longer spans
win if the model proposes several spans for one slot. No utterance-answer table
or destination lookup table is stored. Model proposals are checked against the
schema's intent names, slot types, and allowed categorical values.

Training uses gold labels and prior training states as supervision. Evaluation
uses only predicted prior states. System utterance text is context; system
annotations are never model inputs. The binary's `eval` first predicts and only
then consults gold for its supplementary exact score. The separate Python
scorer calls the pinned upstream SGD implementation for benchmark metrics.

## Sources and sizes

- Dataset: [google-research-datasets/dstc8-schema-guided-dialogue](https://github.com/google-research-datasets/dstc8-schema-guided-dialogue/tree/e852981ae34990f4358979625854259302feaa78),
  commit `e852981ae34990f4358979625854259302feaa78`, **CC-BY-SA-4.0**.
  The fetcher accesses 127 train shards, 20 dev shards, their two schemas, and
  `LICENSE.txt`: **504,306,737 bytes** per successful complete fetch. It uses
  explicit train/dev URLs, not a repository archive. Nonselected dialogues are
  discarded in memory. Per-file sizes and SHA-256 hashes are recorded in
  [reports/data-manifest.json](reports/data-manifest.json).
- Training selection: every train dialogue containing `RideSharing_1`:
  **688 conversations**, **6,852 total user turns**, of which **3,455 target-service
  frames** supply supervision. The span model uses **27,771 user tokens**.
  Retained train JSON: **7,267,443 bytes**. Other-service frames do not supply labels.
- Dev selection: every dialogue whose service list is exactly `["RideSharing_1"]`:
  **45 conversations / 257 user frames**, **220,834 bytes** retained JSON. These
  are the whole selected slice across all 20 official dev shards.
- Metric source: [Google Research SGD metrics](https://github.com/google-research/google-research/blob/5b09c22d73a9d35eb6c5d2a99b95677a45053466/schema_guided_dst/metrics.py),
  commit `5b09c22d73a9d35eb6c5d2a99b95677a45053466`, **Apache-2.0**, **11,209 bytes**; source size and
  SHA-256 are in [tools/metrics-source.json](tools/metrics-source.json), with the
  upstream license alongside the unmodified implementation. Evaluation pins
  NumPy 2.3.3 and fuzzywuzzy 0.18.0, using fuzzywuzzy's pure-Python difflib backend
  (no optional python-Levenshtein package).
- Final model: **7,583,366 bytes**, including 39,824 span features; train-only
  centroid baseline: **7,073,331 bytes**. Parameters and model hashes are in
  [reports/experiment.json](reports/experiment.json). Generated model and dataset
  files live under ignored `scratch/dialogue`, not in the source crate.

## Measured development results

The fixed sample is the first 10 lexicographically sorted dev dialogue IDs:
`1_00123`, `1_00124`, `1_00125`, `1_00126`, `1_00127`, `2_00000`, `2_00001`,
`2_00002`, `2_00003`, `2_00004` (59 user frames). No full-dev failures were used
to tune this batch after its final full-dev evaluation.

| Selection | Model | Official fuzzy joint goal | Exact joint states | Correct intents |
|---|---|---:|---:|---:|
| Fixed sample, 59 frames | Train-only centroids, 0 margin epochs | 26.41 / 59 = 44.76% | 14 / 59 | 59 / 59 |
| Fixed sample, 59 frames | Trained, memory cleared each user turn | 14.00 / 59 = 23.73% | 14 / 59 | 17 / 59 |
| Fixed sample, 59 frames | Trained, carried state | 57.32 / 59 = 97.15% | 54 / 59 | 59 / 59 |
| Complete slice, 257 frames | Train-only centroids, 0 margin epochs | 106.59 / 257 = 41.47% | 52 / 257 | 255 / 257 |
| Complete slice, 257 frames | Trained, memory cleared each user turn | 59.57 / 257 = 23.18% | 58 / 257 | 75 / 257 |
| Complete slice, 257 frames | Trained, carried state | **229.18 / 257 = 89.18%** | **202 / 257 = 78.60%** | **256 / 257 = 99.61%** |

The trained full-slice official average goal accuracy is **228.0216667 / 238 =
95.8074%** (only frames with active gold slots). Categorical joint accuracy is
**243 / 257 = 94.5525%**; noncategorical joint accuracy is **239.34 / 257 =
93.1284%**. Official fuzzy joint goal multiplies per-slot scores within each
frame and averages over frames, so its numerator is a fractional score sum,
not a count of exactly correct frames. Exact joint success is reported separately.
Full totals for every comparison are retained under `reports/`.

The no-memory ablation clears both slots and intent before each user turn while
retaining the immediately preceding system utterance. It uses the same trained
weights; it is a memory ablation, not an independently optimized stateless model.
The centroid baseline uses the same training data and features, with no margin
epochs. Thus training improves full-slice fuzzy joint goal by **47.70 percentage
points** over centroids; carrying state improves it by **66.00 points** over
clearing state in the trained model.

## Reproduce

From this worktree root:

```bash
bash crates/dialogue/tools/reproduce.sh
```

The script fetches only official train/dev, installs pinned metric dependencies
in a local venv, checks formatting, runs targeted clippy, builds this crate,
trains both models, scores the fixed sample and complete selected dev slice,
and audits annotation isolation. All cargo commands explicitly set `TMPDIR`,
use `systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=5s 600`, and compile with `-j 1` (fmt has no `-j`). Runtime and
fetch processes are capped too. No unit tests or test-split evaluations are run.
Generated reports are in `scratch/dialogue/reports/`.

To make predictions without any annotations, save a JSON array containing only
`{"speaker":"USER"|"SYSTEM","utterance":"..."}` objects, one conversation per
invocation, then run:

```bash
TMPDIR="$PWD/scratch/dialogue/tmp" systemd-run --user --scope \
  -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 \
  "${CARGO_TARGET_DIR:-$PWD/scratch/dialogue/target}/release/glassmind-dialogue" predict \
  --schema scratch/dialogue/rides/dev-schema.json \
  --model scratch/dialogue/model-12.json --input path/to/utterances.json
```

## Verification and limits

Targeted release build, formatting check, and clippy with `-D warnings` passed.
The entire reproduction script was then rerun successfully: all six score
reports, both model SHA-256 hashes, and the data manifest matched exactly.
The runtime isolation audit recreated predictions for **all 257 dev user turns**
using stripped utterance-only inputs in separate per-session invocations; every
prediction matched the evaluation output. Annotation-bearing runtime input was
rejected. See [reports/runtime-audit.json](reports/runtime-audit.json).

Not implemented or claimed: multi-service routing, cross-service reference
resolution, unseen schema generalization, requested-slot prediction, normalized
date/number extraction for other services, or whole-benchmark performance.
Noncategorical slot deletion and implicit/system-offered value selection are
also not modeled. Sparse training classes can make the shared classifier reject
training for another service. The final model's smaller exact score shows that
span boundaries and categorical decisions still lose complete states. No LLM,
external API inference, credentials, other agents, or flexion were used.

