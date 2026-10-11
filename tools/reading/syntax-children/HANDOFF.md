# Child representation experiment — rejected, runtime restored

One bounded batch at base `6a346c3080a5dd94d7a6bc7dae32af7f6d969798`.
The candidate improved LAS by20 but failed both frozen accuracy gates. Do not deploy
`rejected.patch` or the candidate model. Both owned runtime files are byte-identical
to the approved base; only this evidence directory remains uncommitted. No commit,
new agent/model call, service, download, official TEST or parent input-case access.
No owned jobs remain. Parent integration and independent probes were not performed.

## Exact experiment

PLAN.md and24 authored contrasts (61 scored core edges,2973 bytes) were SHA-frozen
before runtime edits. Added exactly32 features, eight for each left/right side of
the top2 stack nodes: second-child word/tag/label; head-tag/second-tag/label;
first/second tags; head-tag/first-label/second-label; head-tag/first-tag/second-tag;
other-stack-tag/head-tag/first-tag/second-tag. Missing child values are explicit0.
Namespace100..131; unchanged131072 parser hash buckets. Two bounded index arrays
track second-smallest left and second-largest right attached children in constant
work per attachment. Old features remain an identical prefix. No directional
valency, morphology, epoch, oracle, scoring, beam or temperature change.

An internal validated version selects legacy versus children features. Added
`train_with_child_features`, `Model::version`, and standalone `train --child-features`.
New header/prediction version `ewt-arc-standard-children-v2`; legacy public VERSION,
default train API/CLI, binary magic, header strictness and loader bounds unchanged.
Unknown versions reject. Tagger payload is byte-identical to approved v1.
All runtime input remains strictly forms/text; TRAIN annotations enter only trainer.
128 tokens,16KiB text,128KiB JSONL row,128MiB loader and deterministic greedy decode
remain unchanged. A new model version is not a probability or semantic proof.

## Measured results

Exactly one full allowed TRAIN fit; then source/model/binary/adapters/scorer freeze,
first100 DEV inference, and full2001 DEV inference once. No tuning or retraining.
The unchanged `tools/reading/syntax/parent/audit.py` supplied full independent scores.

| Scope | Approved UAS | Candidate UAS | Approved LAS | Candidate LAS |
|---|---:|---:|---:|---:|
| First100 /2319 words |1834|1810|1738|1726|
| All2001 /25148 words including punctuation |20540|20515|19423|19443|
| Nonduplicate1933 /25011 words |20403|20378|19286|19306|
| TRAIN-duplicate68 /137 words |137|137|137|137|
| Exclude punctuation /22073 words (diagnostic only) |18132|18106|17020|17040|

Required LAS>=19673: FAIL (short230). Required UAS>=20540: FAIL (short25).
All2001 emitted structurally valid trees; zero refusals. No duplicate exclusions.
Root correctness1694->1692. Legacy evaluate.py's internal `baseline_*` fields denote
its old trivial chain baseline, not the approved model; its45%/55% gates are not this
experiment's acceptance criteria. Use the approved comparison and independent scores.

Complete gold-edge and sentence-correctness transitions (1=correct,0=wrong):

| Metric / unit |1->1|1->0|0->1|0->0|
|---|---:|---:|---:|---:|
| UAS edges |19773|767|742|3866|
| LAS edges |18634|789|809|4916|
| UAS exact sentences |951|57|56|937|
| LAS exact sentences |778|50|54|1119|

All duplicate edges137 and sentences68 remain1->1 for both metrics; the nonduplicate
1->1 counts are19636/18497 edges and883/710 sentences. All other transitions above
are nonduplicate. Full per-token and per-sentence ledgers are external, without
exclusions; aggregate groups are in reports/transitions.json.

New authored24:49->51/61 core edges, zero old-correct losses. Existing authored16:
33->33/33 core edges. Combined82->84/94; no gate weakening or relabeling. Parent
frozen supplement was not accessed and remains outstanding, but cannot rescue the
failed full DEV gates. The earlier rejected width4 experiments (17879 LAS and
normalized19428 LAS/20547 UAS) were not retried and their archives were untouched.

## Training, data and resources

Existing read-only UniversalDependencies/UD_English-EWT r2.18 revision
`b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c`, CC-BY-SA-4.0; no new downloaded data.

| File | Bytes | SHA256 |
|---|---:|---|
| TRAIN |15034493|d68e06122a702464c613076523d56740f047e5bbe89dd90ec32737e04d952143|
| DEV |1898574|39239e0a60db3ae68f4b7036189f11b6692741d10ff8240dd91f74f2760d90f8|
| LICENSE.txt |20014|b3d1b0f4c6ae151f7eb78738f46ebd5ee140f8a7f76501ba26af123140d35ae7|

source-provenance.json retains pinned URLs and existing provenance. Authored probes
are newly written development data under this repository's MIT license; they are
neither independent test material nor an answer table used at runtime.

TRAIN12544 sentences/204578 integer words,2614 range rows and37 decimal rows.
Tagger12542 sentences/204284 words, fixed2 ordered epochs; mistakes20605,11984.
Parser12256 sentences/195962 words, fixed3 ordered epochs with static oracle and
predicted tags; mistakes43934,26058,18998. Two over128 input skips (294 words), zero
invalid-tree skips,286 eligible nonprojective skips. Whole-corpus projectivity audit
counts287 nonprojective sentences/8457 words, including one already skipped long
sentence. Exact skip ledger retained externally; no filtering definitions changed.

Training4.579680306s,291808KiB peak RSS. First100 inference0.045753779s/60068KiB;
full2001 inference0.209942583s/59792KiB. Timing includes loading/startup. Outer scope
and every cargo invocation were8GiB/zero swap; every cargo command used on-disk TMPDIR,
systemd user scope, setsid --wait, timeout600s/kill-after10s, -j1 except fmt and4 test
threads. Existing CARGO_TARGET_DIR used. No /tmp storage or background service.

Candidate model13,481,109 bytes (<32MiB), SHA256
`06f84e0df48aa57044cc2f88a872a1bbc8fbad274ed57d406db24ad09a3d38dd`.
Tagger166060 nonzero weights, parser1518991 (103 classes); legacy parser1403194.
Candidate standalone12,241,824 bytes, SHA256
`93ab78d3b6800f91a620e98fe0cbbc918990292ee16a6ebd4cf86b2672e95383`.
Full predictions SHA256
`3bcd37f158c2744a044b4884030ea9aea661abf4cc74b9c962bc325e359bda91`.

## Validation and reproduction

Passed package fmt check, all16 native tests (14 library+2 QA binary), package
all-target clippy -D warnings, release standalone build,40 raw contrasts for both
models,16 input/span contracts for each version, actual-model NaN rejection,
v1/v2 learned-model byte-exact load/save and deterministic bounded inference.
Native tests cover second-child ordering, additive32-feature prefix/buckets,
explicit missing child, both versions, corrupt versions/nonfinite/excessive/zero
weights and bad dimensions. Existing strict loader code remains intact.

The new binary with approved v1 model47d8d245... reproduced all2001 supplied forms
predictions and all2001 raw predictions byte-identically. The approved model itself
remains12,554,724 bytes and unchanged. Main CLI was not edited or rebuilt; parent
owns any later main integration. Default training stays legacy by dispatch; no second
full legacy training run was made. The v1/v2 compatibility results are not evidence
of improved general semantic accuracy.

All exact commands are in reproduce.sh and external candidate/reproduce.log. To
replay this **rejected** experiment only in a disposable checkout of the base:

```bash
git apply tools/reading/syntax-children/rejected.patch
# Retain the supplied non-/tmp TMPDIR and existing CARGO_TARGET_DIR.
OUT=/home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run/reproduction \
  bash tools/reading/syntax-children/reproduce.sh
```

Do not execute that command against an existing model output: the script refuses
to overwrite/retrain it. Its explicit scoped train command uses --child-features.
All cargo, training and inference commands are bounded by the required wrapper.
Reports may be independently rescored without training or inference:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 tools/reading/syntax/parent/audit.py \
  --source /home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source \
  --predictions /home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run/rejected-archive/artifacts/full-predictions.jsonl \
  --out /home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run/independent-audit
```

## Preserved evidence and rollback

Raw inputs/outputs/models/binary/source/patch/logs/ledgers are external under
`/home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run/candidate` and its
immutable `rejected-archive` sibling. Archive contains73 files/98,513,691 bytes,
all individually SHA256/size verified **before** restoring runtime source. Exact
adapter snapshots and reports are included. archive-manifest.json records every
archived file; FREEZE.sha256 records the pre-DEV identity of source, model, binary,
probes, adapter, unchanged scorer and allowed sources. Original candidate is retained.
rollback.json verifies both runtime files against approved6a346c3. Rejected patch
applies cleanly to that base. No runtime change is proposed for integration.

Limitations: observed LAS+20 is far below the preregistered gain and comes with
UAS regression,789 old-correct LAS edge losses and50 exact-LAS sentence losses.
The child family did not meet the intended objective. Parent contrasts/main checks
are outstanding, no independent reproducibility retrain was performed, and no
held-out accuracy claim is made. No second experiment was started.
