# Rejected: one structured first-order graph parser experiment

Base225892adb78afe5d9ea567fe4b08790aa7d93b52. One bounded fit, no second configuration
or DEV tuning. Runtime source restored exactly after verified archive; do not
integrate rejected.patch/model. Coordinator owns independent review and integration.
Only tools/reading/syntax-graph remains uncommitted. No agents, new model calls,
credentials/API keys, commits, services, corpus downloads, official TEST, hidden
items/generators or parent case-input access. Parent reports were read as requested.
The required algorithm paper was consulted online; no new training data acquired.
Launcher controls model/login settings; this worker did not change or verify them.

## Frozen implementation

PLAN.md and24 authored probes frozen before editing. New syntax_graph.rs implements
learned labeled arc scores, exact Chu-Liu-Edmonds cycle contraction/expansion, and
exact enumeration of possible unique root children. An already single-root
unconstrained optimum returns immediately. Independent root/tree validation follows
inference. Matrices finite, <=129 nodes, missing edges explicit; deterministic
strict-greater ties. DECODER.md gives optimality argument, complexity and tests.

One flat2^21 hashed scalar parameter vector;28 shared unlabeled features plus12
label-specific features for each arc. Lexical lowercased hashes, predicted UPOS,
direction/distance and adjacent tag context. Exact inventory/namespaces fixed in
PLAN; no sentence/entity tables. Label maximum computed independently per arc;
root label iff artificial-root edge. Structured gold-tree minus predicted-tree
perceptron, one update clock per sentence, average pre-update vectors, exactly
three ordered TRAIN epochs. No search normalization, beam or child-state feature.

Approved v1 tagger copied without retraining; exact1328492-byte serialized tagger
payload SHA0409e47e9376ee1593480461fa35bbe7cfa1c21f6d74fce7ba9bad167dbba39f
matches v2. Predicted tags during both TRAIN and inference. No gold-tag inference.
TRAIN report tag_epochs0. Parser weights trained from zero, no rejected weights.

Existing Model::load/save/predict/infer dispatch on explicit validated version:
legacy ewt-arc-standard-v1 or new ewt-labeled-graph-v2. Binary magic unchanged;
header version determines strict table dimensions; unknown versions reject.
New train_graph API and standalone train --graph-tagger legacy-model.bin flag.
Default train API/CLI remains legacy. Main CLI/QA/manifests/scorer untouched.
128tokens/16KiB text+forms/128KiB JSONL row/128MiB loader limits retained.
Actual new model9715189 bytes (<32MiB), SHA
5168e505cfd1a11502a707e403e482a83fde821aaaed270412990d24bec19781.

## Observed gates — failed

Unchanged tools/reading/syntax/parent/audit.py; all integer words including punct.
One fixed first100 inference then one full2001 inference after FREEZE.json.
The scorer's historical45%/55% assertions are NOT this experiment's acceptance.

| Scope | Words | Approved LAS | Graph LAS | Approved UAS | Graph UAS |
|---|---:|---:|---:|---:|---:|
| First100 |2319|1738|1546|1834|1713|
| Full2001 |25148|19423|17900|20540|19732|
| Nonduplicate1933 |25011|19286|17763|20403|19595|
| TRAIN-duplicate68 |137|137|137|137|137|

LAS delta-1523, below19673 by1773: FAIL. UAS delta-808: FAIL.
All2001 forms outputs valid single-root trees, zero refusals. Root correctness
1694->1682. Exact LAS trees828->583; exact UAS1008->794.
Duplicates remain in all denominators and all137 edges/68 sentences stay correct.

| Full correctness transitions | right->right | right->wrong | wrong->right | wrong->wrong |
|---|---:|---:|---:|---:|
| LAS edges |16554|2869|1346|4379|
| UAS edges |18247|2293|1485|3123|
| Exact LAS trees |551|277|32|1141|
| Exact UAS trees |716|292|78|915|

All25148 edge transitions and2001 sentence transitions retained, including unchanged
ones, with gold/old/new and duplicate flag. Aggregate transitions.json separately
reports duplicates/nonduplicates. Authored24 cases:92->93/117 labeled edges,
91 retained,1 lost,2 gained,23 still wrong. Local preservation therefore also fails;
authored-changes.json records each transition. Parent independent probes untouched
and unrun; no independent semantic-preservation claim. They cannot rescue corpus
failure. No QA-answer or raw-tokenizer accuracy claim follows from gold-form LAS.

Prior rejected hypotheses recorded in PLAN: raw beam17879 LAS, normalized19428,
children19443 against approved19423. This is a different objective/representation,
but this particular frozen graph feature/training configuration also regresses.
No second fit, additional epochs or post-DEV feature changes attempted.

## Validation and resources

Passed10 focused syntax library tests (6 existing +4 graph/compatibility tests),
including exhaustive maximum-score comparisons on500 independent matrices with
1..5 tokens, explicit nested contractions, multiple-root unconstrained optimum,
negative weights, ties/determinism, nonfinite/disconnected rejection, v1/v2 tiny
model roundtrip/unknown version/nonfinite/trailing bytes and128/129-token limits.
Existing loader-dimension and input/span tests retained. Actual two-model roundtrip
and exact frozen-tagger equality passed in an additional targeted test invocation.
The optional actual-model test is one of the10 tests; its real payload branch ran
in this later invocation. The500-matrix oracle enumerates every possible parent
assignment and validates trees independently of contraction.

Package fmt, all-target clippy -D warnings, release reading-syntax build passed.
Two initial clippy failures (needless range loops, one in tests) fixed before fit
and preserved. No observed decoder or fitting failure. No workspace/main build
performed; parent owns any integration and rejection makes it unnecessary.

Candidate binary with approved v1 model reproduced all2001 supplied forms outputs
and all2001 supplied raw outputs BYTEIDENTICALLY. Both models save/load byte-exact.
Graph raw2001 predictions all valid with exact UTF8 spans and source coverage.
Ten runtime row contracts/version passed:8 refusals plus128-token and Unicode
positive cases; oversized128KiB JSONL row rejected. This is raw structural/span
validation, not gold dependency accuracy after tokenization.

TRAIN12544/204578 words:12542 eligible/204284 words; zero invalid trees;
286 previously skipped nonprojective trees now included. Two overlength skips:
zero-based sentence6779 length159,7506 length135 (294 words). Full ledger in
eligibility.json. Fixed3 epoch labeled-edge mistakes75681,53905,42352.
Tagger never updated. No denominator or epoch reduction.

Training48.6606s/176492KiB peakRSS. First100 inference0.198557s/16664KiB;
full2001 inference1.440086s/16856KiB. Includes process/model loading. Binary12538168
bytes. Exact commands/rusage in candidate/*-resources.json; these short runs are
not steady-state throughput estimates. All cargo invocations set persistent TMPDIR,
used supplied CARGO_TARGET_DIR, systemd user8GiB/0swap scope, setsid --wait,
timeout600s/kill-after10s, -j1 except fmt, --test-threads4 for tests. Training capped
600s; each full inference capped600s under the verified outer8GiB/0swap scope.
No /tmp storage or lingering owned jobs.

## Provenance, archive and reproduction

Existing EWT r2.18 revisionb7711cce01cdd4f5fcc0a8199b8a50d951b16c0c,
CC-BY-SA-4.0; TRAIN15034493 bytes +DEV1898574 bytes, hashes in NOTICE.md and
provenance.json. New authored probes24 cases3136 bytes plus forms-only projection
1236 bytes, MIT; identities in pre-edit.json. No new downloaded training data.

Run=/home/gsc/.cache/glassmind/continuous-20261010/graphsyntax-run.
Archive run/frozen-candidate contains original/final source, baseline and candidate
models/binaries, full inputs/predictions/ledgers, adapters, patch including all new
files, commands, failures, source provenance and frozen identities. Size/SHA256
manifest is run/frozen-candidate-manifest.json; ARCHIVE.json pins its own identity.
Every archived file verified BEFORE rollback. rollback.json proves exact restored
source. FREEZE.json and prefit.json reverified unchanged after evaluation.
No runtime patch is proposed for integration.

Replay only in a disposable checkout at the stated base with sibling graphsyntax-run
and udsyntax-source directories. Preserve the assigned existing target and non-/tmp
TMPDIR. Copy supplied approved baseline bundle into fresh run/baseline; do not
reuse a run that contains prefit.json/model.bin/FREEZE.json. Apply only production
hunks from rejected.patch (the tools files already exist in this handoff):

```bash
git apply --include='crates/reading/src/*' tools/reading/syntax-graph/rejected.patch
bash tools/reading/syntax-graph/check.sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/reading/syntax-graph/evaluate.py prepare
bash tools/reading/syntax-graph/fit.sh
SYNTAX_GRAPH_MODELS="$PWD/../graphsyntax-run/baseline/model.bin:$PWD/../graphsyntax-run/candidate/model.bin" \
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind-reading -j1 \
--lib supplied_models_roundtrip_when_requested -- --test-threads 4
PYTHONDONTWRITEBYTECODE=1 python3 tools/reading/syntax-graph/evaluate.py evaluate
```

Read-only rescore requires no fit/inference:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 tools/reading/syntax/parent/audit.py \
 --source ../udsyntax-source \
 --predictions ../graphsyntax-run/frozen-candidate/candidate/full-predictions.jsonl \
 --out ../graphsyntax-run/independent-rescore
```

Limitations: graph scores are not probabilities or semantic proof; exact decoding
only optimizes this learned first-order objective. Labels and tags retain the
approved TRAIN inventory. No cross-track/held-out/parent-independent accuracy
result, no independent retraining reproduction and no claim beyond observed data.
The architectural experiment is rejected, not evidence that every graph parser
would regress. The parent continuing goal is unaffected by this finite handoff.
