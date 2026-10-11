# Everyday intents and typed chat: first batch

The standalone package/binary is `glassmind-everyday`. No existing source crate
is changed. Its library exposes `IntentModel`, `Prediction`, `Conversation`,
`Reply`, and a JSONL benchmark adapter. No runtime LLM or flexion dependency.

## Data and isolation

Source: [CLINC150, Larson et al. (2019), official clinc/oos-eval repository](https://github.com/clinc/oos-eval),
revision `828f8093932c8fe6ca7936c3d2e52903b1c523de`, `data/data_full.json`.
License: [CC BY 3.0](https://github.com/clinc/oos-eval/blob/828f8093932c8fe6ca7936c3d2e52903b1c523de/LICENSE).
Upstream file size from HTTP metadata: **2,495,390 bytes**.

The downloader consumes the allowed prefix (`oos_val`, `val`, `train`) and
stops at the first disallowed section header **before consuming its items**.
It never saves the complete upstream file or reads a test item. The prefix
including the stop header is 1,911,799 bytes. The later `oos_train` section is
not accessed: training uses **15,000 in-scope rows**, 100 per class, rather
than claiming to use all 15,100 official training rows.

| Extracted JSONL | Rows | Bytes | SHA-256 |
|---|---:|---:|---|
| train | 15,000 | 1,165,484 | `f27778e9c16448d040d9c364a1b369493711865419613a277270a54781ea87c2` |
| val | 3,100 | 239,360 | `39afa094ba8495e9b8b30595394937bdd7b5b8e6dcdba71a9a380047759c88f9` |

Validation comprises 20 examples for each of 150 intents plus 100 OOS
examples. The official training/validation texts overlap exactly in three
cases; the official splits and totals are preserved. Source metadata and
hashes are written to `scratch/everyday/clinc/source.json`.

## Model and selection

Training-only vocabulary and IDF; word unigrams, adjacent bigrams, and
within-word character 3–5-grams; normalized sparse TF-IDF vectors. A
nearest-centroid initializer is refined by a multiclass passive-aggressive
margin learner with a deterministic shuffle and averaging over the last
half of 20 epochs. Character weight is 0.3 and minimum document frequency 2.
The final model has 31,435 features and 150 learned labels. It stores weights,
feature names and counts, not examples or an answer lookup table.

Validation only selects minimum score/margin thresholds maximizing
`right - 2 * wrong`; an in-scope refusal has utility zero, a correct OOS
refusal counts as right, and an accepted OOS example counts as wrong.
Final thresholds are score >= 0 and margin >= 0.30. Reported confidence is
the empirical correctness fraction in a validation margin bin, with its
support count, **not** a posterior probability or an independent calibration
estimate. `intent --explain` lists signed feature contributions.

The fixed sample is the first three original-order validation rows per label:
450 in-scope + 3 OOS = 453. It is unchanged across model variants. Small
sample comparisons also tried word-only margin learning, character weights
0.15 and 0.5, and 40 epochs. Results were mixed (sample utility 377–387 versus
384 for the retained default); these small differences did not justify a
default change. The sample has only three OOS examples, so the final rejection
policy uses all official validation rows.

## Measured dev totals

These are **development/selection results**, not test results. Model weights
never use validation labels, but thresholds and development choices do.

| Model / split | Right | In-scope refusal | Wrong | Raw in-scope top-1 | Correct OOS refusal |
|---|---:|---:|---:|---:|---:|
| Word centroid, fixed 453 | 336 | 90 | 27 | 379/450 | 1/3 |
| Word+char margin, fixed 453 | 402 | 42 | 9 | 425/450 | 2/3 |
| Word centroid, full val, full calibration | 2,282 | 598 | 220 | 2,495/3,000 | 67/100 |
| Word+char margin, full val, full calibration | **2,653** | **364** | **83** | **2,780/3,000** | **82/100** |

The raw in-scope improvement is 83.17% -> 92.67% (+9.5 percentage points).
The final policy is 85.58% right overall; of 2,654 accepted in-scope proposals
or accepted OOS inputs, 2,571 are correct (96.87% accepted precision).
Both full-val rows use the same objective and full-val threshold calibration.

Serialized final model: **25,524,518 bytes** (JSON), SHA-256
`a84fd59c3e49db08300f61ee33f6d73e64a8ea18109102b01685333b95936408`.
The centroid model is 8,181,032 bytes. Generated data, reports and models stay
in ignored `scratch/everyday`; they can be rebuilt from the pinned source.

## Conversation behavior

`Conversation` holds a name, preferences keyed by category, a focused slot,
the previous exact user utterance, and the latest correction with old/new
values and source turns. Explicit statements write facts. Queries retrieve
typed state; missing information stays unknown. Examples:

```text
My name is Ada.
My favorite drink is tea.
Actually, it's coffee.
What about that?
And my name?
What did I just say?
```

`call me Dr. Kim` accepts properly cased name shorthand; lowercase or
ambiguous shorthand should use `my name is ...` instead. An optional session
file persists state after each turn. `/reset` clears it; `/quit` exits.
Greeting and identity responses are available. Other recognized requests
return intent proposals and explicitly say an external service is not
connected: the binary never pretends to book, pay, call, or set a timer.

The synthetic CLI memory smoke passes **41 checks, 0 failures**, including
corrections, cross-slot follow-ups, previous-message recall, Unicode values,
session reload, reset, missing information and ambiguous call requests.
This is authored development coverage, **not** a multi-turn benchmark result.
No SGD/MultiWOZ track, unrestricted dialogue understanding, external action
executor, or core `glassmind` CLI integration is implemented in this batch.

## Exact reproduction and use

From the workspace root:

```bash
bash crates/everyday/tools/reproduce.sh
```

That script fetches only permitted data; runs targeted fmt/build/clippy;
reproduces both sample and full official-val rows; and checks memory through
the actual CLI. Every cargo invocation inherits an explicit non-`/tmp`
`TMPDIR`, runs in an 8 GiB/no-swap systemd scope with a 600-second process-group
timeout, and uses one build job (fmt has no job flag). `setsid --wait` is
necessary to propagate exit status and wait for the scoped child. No unit
tests or held-out evaluation are run.

Individual build/check commands:

```bash
mkdir -p "$PWD/scratch/everyday/tmp"
TMPDIR="$PWD/scratch/everyday/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 cargo build -p glassmind-everyday --release -j 1
TMPDIR="$PWD/scratch/everyday/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 cargo clippy -p glassmind-everyday --all-targets -j 1 -- -D warnings
```

After reproduction, use the actual target directory (the script honors
`CARGO_TARGET_DIR`; when unset it uses `./target`):

```bash
binary="${CARGO_TARGET_DIR:-$PWD/target}/release/glassmind-everyday"
"$binary" intent --model scratch/everyday/intent.json --explain 'Set a timer for ten minutes'
"$binary" chat --model scratch/everyday/intent.json --session scratch/everyday/session.json
"$binary" eval --model scratch/everyday/intent.json --dev scratch/everyday/clinc/val.jsonl --per-label 0
```

For integration, add a path dependency on `crates/everyday`, deserialize and
`validate()` an `IntentModel`, then call `predict(text)` or
`conversation.respond(text, Some(&model))`. `adapter::read_examples`,
`fixed_sample`, and `Totals::observe` expose the same evaluation semantics as
the CLI. `chat --json` emits one typed reply per line for a process adapter.
