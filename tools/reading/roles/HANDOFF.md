# Reading roles — REJECTED; evidence only

One bounded experiment at base `eb22519f20325e72549a103cf99180ea0dbc2ba3`.
**Runtime restored exactly. Do not deploy this candidate or apply its patch in
production.** Only new `tools/reading/roles/` evidence remains. No commits,
agents, LLM calls, services, downloads, credentials, API keys or held-out access.
All owned jobs finished. Main CLI, loop, shared manifests, README files,
probability implementation, existing scorers and historical artifacts unchanged.

## Parent review

Original20-file/277,065-byte submission archived and hash-verified before
worktree removal. Temporary login link removed; worker completed normally.
Parent verified82 provenance hashes, independently recomputed all10570 official
EM/F1 scores and transitions, and recalculated the CAL threshold. No repeated
inference or training. Runtime diff is empty; rejected patch applies cleanly.

**Transition correction:** the worker helper included gold variants that normalize
to empty, unlike the official scorer. Three unchanged refusals were mislabeled
right→right. Correct counts are363 right→right and8273 refused→refused; all
other transitions and the published aggregate scores are unchanged. The table
below preserves the original worker report. Authoritative corrected transitions
are in [parent-review.json](reports/parent-review.json), reproduced with
`SQUAD_SOURCE=... ROLES_RUN=... ROLES_BASE=... python3 tools/reading/roles/parent-audit.py`.
The original candidate remains rejected under its unchanged gates.

## Exact experiment

[PREREGISTER.md](PREREGISTER.md) predates implementation and DEV evaluation.
Inspected first14 original FIT rows (retained in reports/train-inspection.json).
Hypothesis: candidate-relative lexical anchors help distinguish argument roles
that symmetric overlap cannot represent. One feature family and one setting,
no search, no dev retuning, no threshold change after evaluation.

[rejected.patch](rejected.patch) preserves the exact evaluated modifications:
optional version2 `Model::train_roles` and version dispatch in qa.rs; trainer
`--feature-version 2`; two focused tests. Appends hashed stem-matched question
anchors conditioned on candidate side/distance, question order, auxiliary and
morphology, nearest-anchor order and context, local negation agreement, and
question-head/answer-boundary conjunctions. Same65536-f32 inspectable linear
weights, optimizer/rate cap0.1, averaging, five passes and candidate search
(top3 sentences, 1..10token contiguous spans). Version1 bypasses new features.
Input limits remain64KiB/2048 context tokens/2KiB question; finite-score and
UTF-8 source/evidence guards retained. No entity/predicate/question tables,
pretrained embeddings, runtime LLM or runtime gold labels.

Reconstructed the original TRAIN selection independently and verified all four
FIT/CAL input/label files byte-identical plus exact selection JSON equality:
4000 FIT/64 article hashes,1000 CAL/next16 article hashes;3214 representable FIT
rows in every pass, identical to v1. No expanded data. CAL maximum-coverage rule
unchanged: >=50% normalized exact, >=30 accepts, grouped ties. V2 selects86,
43right43wrong, threshold0.20381558; v1 selected148,74right74wrong,
threshold0.18979406. Audit recomputed all CAL prefix groups independently;
complete frontier retained outside git. These are selection estimates, not
probabilities or entailment confidence.

Model and binary SHA frozen before first200 SHA256(ID), then full10570 once.
Official normalized EM/F1 imported unchanged through existing qa/run.py,
including its pinned official scorer hash check. Exact ordered input equality,
unique IDs, every scorer ID and full count asserted before reporting.

## Before/after and decision

| All10570 DEV | Frozen v1 | Candidate v2 |
|---|---:|---:|
| Raw exact correct |2532|2631|
| Raw EM % |23.9545884579|24.8912015137|
| Raw F1 % |34.5184250480|35.9725577883|
| Selected correct |815|694|
| Selected wrong |791|498|
| Refused |8964|9378|
| Selected exact precision % |50.7471980075|58.2214765101|
| Coverage % |15.1939451277|11.2771996216|
| Selected EM % |7.7105014191|6.5657521287|
| Selected F1 % |8.7001900898|7.2320432457|

**Reject:** raw exact2631<2638 (seven short), selected correct694<815.
F1 and precision pass their gates, but that cannot rescue the two failed gates.
Raw gain+99 exact/+0.936613 percentage points and+1.454133 F1 is a rejected
partial gain, not a shipping improvement. No narrower gate or extra candidate.

First200: raw EM23.0→21.5%, F131.2087565→32.8651316%; selected
16right17wrong167refused→10right7wrong183refused. This sample did not change the
frozen representation, settings, calibration rule or required full evaluation.
[Full metrics](reports/full.json), [sample](reports/sample.json),
[validation](reports/validation.json).

## All transitions

Raw: right→right1568, right→wrong964, wrong→right1063, wrong→wrong6975.
7393 raw strings changed;5256 wrong→wrong strings changed. Thus964 old correct
lost,1063 new correct,964 newly wrong from previously correct proposals.

| Selected before → after | Count |
|---|---:|
|right→right|366|
|right→wrong|6|
|right→refused|446|
|wrong→right|7|
|wrong→wrong|125|
|wrong→refused|659|
|refused→right|324|
|refused→wrong|367|
|refused→refused|8270|

Selected loses452 old correct, gains331 new correct;373 become newly wrong from
right/refused, with18 additional changed wrong→wrong answers.1828 selected
strings changed. Every one of10570 IDs, old/new raw and selected text and both
transition labels is retained in `RUN/full.transitions.json` (not only changes),
SHA256 `42e4b0e1654388a5387b3fcc084041f179838a45b8e13ae3a7f6e329a14aa8de`.
The corresponding200-row transitions are also retained. No losses omitted.

## Independent predeclared probes and limits

[All18 probe cases](probes.json) were frozen before training/dev;
[all outputs](reports/probes.json) include v1/v2 raw and selected behavior.
Selected expected behavior10/18→12/18. These are authored diagnostics, not a
benchmark claim. Every candidate failure:

- Both object-role positives (Nora thanked Felix / Felix thanked Nora): correct
  raw object but refused. Reversed unsupported counterparts refuse.
- Mismatched predicate: painted bridge still accepts2008 when asked when demolished.
- Negation: never demolished still accepts2008 when asked when demolished.
- Person kind: “Mira met Soren in Oslo” / “Who did Mira meet?” proposes whole
  sentence, then refuses instead of selecting Soren.
- Number kind: “Mira carried three crates” proposes three but refuses.

Baseline failures also remain fully recorded: both object positives, reversed
unsupported subject, painted/negated predicates, person kind, unsupported buy
relation and supported buy-date counterpart. Candidate handles the two Unicode
positives and positive demolition/negation/date counterparts, but cannot prove
predicate/role/negation support. The coarse suffix stemmer, first-match anchors,
hash collisions, bounded spans and limited FIT signal remain substantial limits.
SQuAD TRAIN has no genuine unanswerable supervision. Source occurrence is not
logical proof or entailment; all answers explicitly unverified. No automatic
factual talk/knowledge use, no claim of calibrated confidence.

## Validation, failures and identities

Release build and formatting passed.8 QA tests passed (six existing plus two
new), including model validation, finite overflow refusal, input bounds,
UTF-8 provenance and identical v1 candidate/feature prefixes.2 existing byte
probability tests passed. Initial probability test used the library target and
ran0 tests; corrected to `--bin reading-probability`, actually2 passed.
Full10570 v1 replay is byte-identical to frozen outputs, SHA256
`37db70ae0c949a89427a707c37812a666d9507a2192bcb75ed3e4145d3a598a7`.
Probability source byte equality to HEAD verified. `git diff --exit-code --
crates/reading` and `git apply --check tools/reading/roles/rejected.patch` pass.

**Clippy failed** with14 indexing/slicing violations in experimental alignment
code. Exact diagnostic retained in RUN/clippy.log; no post-dev code fixes or
second model run. Rejection already required by metrics. This patch is evidence,
not production-ready code. No main CLI integration or broad workspace suite was
run; parent integration/verification would be required for any future accepted
candidate. Do not publish this rejected model as accepted.

| Artifact | Bytes | SHA256 |
|---|---:|---|
|original v1 model|873881|`b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5`|
|rejected v2 model|1077405|`b33fd772f5c6f57896c99bee5754eb91eef7d0419178514d2488a0ae1879d6ad`|
|frozen reading-qa binary|11686896|`c1a53debe43020960d60f083e2010bc75ef04958de4bf993e2f88f8af98fc4c2`|
|evaluated qa.rs|29369|`7c0a987178130c472405af900b9caa25b7c442c72eeb83e7895cae86787d39a1`|
|evaluated reading-qa.rs|2607|`cf8614c34461aae48dc454de835e179cff6c7ab8c38346b0987f166bb116d88e`|
|rejected patch|14478|`6010fee4a3ea775d49d467e4dd8352f6b847c4dd446b699d1a6616c409ca8039`|
|v2 full outputs|1958778|`4b6aff8bfcaf33a2360f547e3c89030cf07e73d307a8e0280559d922eac507ef`|

Large artifacts outside git:
`RUN=/home/gsc/.cache/glassmind/continuous-20261010/qaroles-run/roles`.
Original baseline directory untouched. RUN contains exact evaluated sources,
original sources, frozen binary/model, FIT/CAL inputs, all outputs, labels,
full/sample transitions, CAL frontier and build/test/train/evaluation logs.
[provenance.json](reports/provenance.json) records exact hashes/bytes, including
source, baseline, binary, model, scorer, all inputs/results and report files.

## Data and licenses

Zero new external data bytes; zero downloads/dependencies. Reused readonly
SQuAD1.1 TRAIN/DEV source `eee5fdbf62f8613a7812b03419e6b29617b74fd1`,
CC-BY-SA-4.0; repository and pinned official v2 scorer MIT.
Source: https://github.com/rajpurkar/SQuAD-explorer/tree/eee5fdbf62f8613a7812b03419e6b29617b74fd1
and https://rajpurkar.github.io/SQuAD-explorer/ .
TRAIN30,288,272 bytes/87599questions; DEV4,854,279 bytes/10570questions;
total35,142,551 bytes. Derived examples retain source license. Copied original
FIT training3,573,508 bytes/FIT context-question3,423,180 bytes;
CAL context-question869,071 bytes/CAL labels1,029,325 bytes.
The18 probes are original diagnostic text. Code MIT. Scorer SHA256
`710840ce2c2c61716334c056777032f24660b83eb9869a451036e6a24d8103c4`.
No held-out/test data, hidden tests/generators, private historical logs or model
predictions downloaded or read.

## Reproduction

For deliberate reproduction of this rejected candidate in a disposable checkout:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/qaroles-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/qaroles-run/target
export ROLES_BASE=/home/gsc/.cache/glassmind/continuous-20261010/qaroles-run/baseline
export SQUAD_SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-source
export ROLES_RUN=/home/gsc/.cache/glassmind/continuous-20261010/qaroles-run/roles-replay
git apply tools/reading/roles/rejected.patch
bash tools/reading/roles/reproduce.sh
# Expected lint failure on the exact rejected patch:
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  cargo clippy -p glassmind-reading --lib --bin reading-qa --no-deps -j1 -- -D warnings
# Restore runtime after reproducing:
git apply -R tools/reading/roles/rejected.patch
python3 tools/reading/roles/audit.py
python3 tools/reading/roles/provenance.py
```

Script honors fresh RUN but report files are regenerated in this owned directory;
use a disposable checkout to preserve this original submission. Actual bounded
commands ran individually, not a second full script replay. Every cargo command
set disk TMPDIR,8GiB/zero swap systemd scope, setsid --wait,600-second timeout,
kill-after10s,-j1 except fmt; tests use4threads. All training/evaluation also
scoped. Outer worker scope verified8GiB/zero swap. No jobs left running.

Final decision: **REJECT**. Runtime exactly restored; retain evidence for parent.
