# Freshness / source reliability — rejected bounded experiment

**Do not integrate the runtime patch.** One contiguous-BIO hypothesis was tested and rejected. Full dev loses15 correct joint turns and17 exact frames, repairs no scored assertion, and leaves both inherited copy errors unchanged. All `crates/dialogue` edits have been restored byte-for-byte to base `38ef238c560d11f0627adb14a946ceb935bc4288`. Only this new `tools/dialogue/freshness/` directory remains uncommitted. No commits or other ownership changes.

## Hypothesis, TRAIN evidence and execution limitation

[EXPERIMENT.md](EXPERIMENT.md) was written before runtime edits. Initial TRAIN inspection:3174 conversations/34022USER turns;12037 annotated noncategorical/categorical input spans identified through full TRAIN schema,8286 multiword. No repeated same-slot annotated spans within a frame, so “latest versus longest correction” was not pursued. Fixed conversation hash split: SHA256(dialogue_id) interpreted as integer modulo5==0 gives642 calibration/2532fit. No classifier training or threshold fitting occurred.

Frozen token diagnosis first100 conversations sorted by SHA256(dialogue_id):80fit/20calibration,1063USER turns. No changed decoded spans on gold-active frames. We nevertheless ran fixed DEV sample/full as falsification and expanded diagnostics to all TRAIN, without changing the candidate. This deviates from the preregistered conditional TRAIN gate; it is disclosed, not reinterpreted as evidence supporting the candidate. The frozen classifiers already trained on both partitions, so these partitions are diagnostic groups, **not independent probability calibration**.

**Completed all-TRAIN follow-up:** all3174conversations/34022USER turns,2532fit/642calibration, also yield zero changed raw decoded spans on gold-active selected-service frames. This reinforces the absence of TRAIN evidence for this frozen-classifier decoder change. These are token diagnostics, not full TRAIN session scores; inactive or nonselected gold services do not contribute. [train-full.json](reports/train-full.json) and [selection-full.json](reports/selection-full.json) retain results and exact IDs. No calibration claim is made.

Candidate: optional multi-model v3 `contiguous_spans=true`; an I token extends the most recent same-slot span only if adjacent to its previous predicted slot token. Frozen classifier labels, longest-span selection, routers, intents, references, offers and existing v2 punctuation boundaries remain identical. Legacy `DialogueModel::advance` always uses the old decoder. No values, cities, service IDs, lexical conditions or dev-derived entity lists enter the rule. `rejected.patch` includes the experimental runtime and offline utterance-only token-trace binary, solely for reproduction.

## Measurements and complete audit

| Scope | Joint baseline→candidate | Exact frames baseline→candidate |
|---|---:|---:|
| Fixed first10 conversations |35→35/91|48→48/108|
| Full179 conversations |610→595/1119|716→699/1218|
| Multi-service54 |153→153/492|248→248/591|
| Single-service125 |457→442/627|468→451/627|

Candidate595right/474wrong/50refused. Joint transitions595right→right,15right→wrong,459wrong→wrong,50refused→refused. No wrong→right frame or joint transition. Routing977/1119 and active intents unchanged;3357predicted frames retained. Fuzzy joint sum784.7596→774.9148/1218; do not present these as calibrated probabilities.

[audit.json](reports/audit.json) enumerates **every30 changed slot-turn assertions**:20correct→wrong,7wrong→wrong,3held-unscored→held-unscored. Thus27 newly altered wrong assertions, including20 newly introduced errors. [changed-event-traces.json](reports/changed-event-traces.json) explains all seven initiating events. Direct source matching is reported only as grounding, never proof of the intended current referent. No changed held states are credited as correct.

The inherited baseline audit is explicitly joined by conversation/turn/service/slot: all4 boundary-repaired scored assertions stay correct; **all7 remaining wrong slot-turns stay wrong**. The original six held boundary changes remain unscored. No transfer suppression occurred.

Actual causes from frozen traces:

- Gap bridging currently recovers true spans when the classifier marks internal words O. In six newly harmed events, gaps include function words inside dates/names and an address component. For example `5th/B:date of/O this/I:date month/I:date` becomes only `this month`; full address/name fragments similarly truncate. The seventh event merely turns one incorrect destination into another incorrect fragment.
- Stale referent `17_00022`: turn4 correction receives all O labels, including its new location. Source state therefore retains the old location, which is later copied. BIO adjacency cannot help when the new referent has no span proposal.
- Truncated source `17_00038`: turn2 gets `San/B:city Fran/O`; there is no later I token to join. Contiguity cannot recover the missing suffix. These dev values appear only in offline evidence, never runtime rules.

[inherited-traces.json](reports/inherited-traces.json) retains the relevant frozen utterance/token traces. The next substantive mechanism would need new correction/span proposal evidence; generic gap closure is contradicted by this batch. No second hypothesis was attempted.

## Validation and reproduction

Existing eight focused Rust library tests pass, including reset/provenance/annotation invariants. Release build and crate fmt pass. All-target clippy passes after replacing three guarded diagnostic argument indices with slice destructuring; no inference behavior changed. No new candidate-specific unit tests were retained because the candidate was rejected. Full179 reversed-conversation replay is identical; annotation-bearing runtime input rejects. Each CLI conversation uses a fresh session, with reset covered by existing tests.

The candidate executable with **optional mechanism absent** reproduces frozen v2 full predictions byte-for-byte. Legacy45 predictions likewise reproduce byte-for-byte:202/257strict,256/257intents,89.17509727626459%fuzzy. Every pre-experiment source/evaluator/model hash was rechecked after rollback. [validation.json](reports/validation.json), [rollback-verification.json](reports/rollback-verification.json), [before-hashes.json](reports/before-hashes.json) and [frozen-weights.json](reports/frozen-weights.json) record checks and unchanged weights. No scorers or evaluation definitions changed.

Run in an isolated clean checkout at the base above; the script temporarily applies the **rejected** patch and reverses it on exit:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/freshness-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/freshness-run/target
export SGD_ROOT=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full
export FRESHNESS_BASELINE=/home/gsc/.cache/glassmind/continuous-20261010/freshness-run/baseline
bash tools/dialogue/freshness/reproduce.sh
```

Commands were executed individually in this batch; the consolidated script was syntax checked, not rerun end-to-end. Original artifacts are in `$TMPDIR/freshness/`; script output goes to `$TMPDIR/freshness-reproduction/`. Evaluated binaries were copied to `evaluated-{dialogue-multi,dialogue-span-trace,glassmind-dialogue}` before rollback. The token-trace binary predates the diagnostic argument-parser-only lint fix; scored dialogue-multi source is unchanged by that fix. The existing target directory contains rejected candidate build artifacts; use the frozen model or rebuild restored source, never deploy the candidate model.

Every cargo invocation set persistent TMPDIR and used `systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`, `-j1` except fmt, tests `--test-threads 4`. TRAIN trace/evaluation inference was similarly scoped. Outer scope verified8GiB/0swap. No agents, LLM calls, credential reads, API keys, services, downloads, `/tmp`, commits, hidden/test/held-out content, old private logs, or scorer edits. No owned background process remains at handoff. Host model/login cannot be independently attested by this worker; no alternate model/authentication was invoked.

## Source, license, bytes and remaining limits

**New external data:0bytes; downloaded:0bytes.** Reused SGD `google-research-datasets/dstc8-schema-guided-dialogue` at `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0. All150 allowed raw train/dev/schema/license files rehashed:504306737bytes. Selected TRAIN45291003bytes; selected DEV1207178bytes; full TRAIN schema75764bytes. Selection remains the baseline3174TRAIN and179DEV conversations. Derived data-bearing reports retain CC-BY-SA-4.0; no new dependency or lexicon. Full file list and hashes: [data-provenance.json](reports/data-provenance.json). Unchanged upstream metric Apache-2.0 revision `5b09c22d73a9d35eb6c5d2a99b95677a45053466`,11209bytes.

[artifacts.json](reports/artifacts.json) records exact model, predictions, trace inputs/outputs and executable sizes/SHA256. [candidate-source.json](reports/candidate-source.json) freezes candidate source and dependencies; rollback leaves baseline runtime source identical. Restricted three-service dev evidence only. This batch produces **no accepted capability improvement**; the deliverable is a reproducible negative result and a causal account of why the apparent decoder bug does not fix the remaining freshness failures.
