# REJECTED — width-4 cumulative legal log-softmax syntax search

Do not integrate rejected.patch. This single frozen candidate fails the required
LAS gate:19423->19428 (+5), below19673 (+250). UAS20540->20547 (+7).
Both owned runtime files were restored byte-for-byte after source/binary/patch/
results archival and hash verification. Only tools/reading/syntax-normalized is
uncommitted. No main CLI, QA, coding, quantities, manifest, scorer, tokenizer,
training or default inference changes remain. Root coding repair8aeb5ac was untouched.

Base source8aeb5ac1a888dcea571d4c0710e059005442b215.
Run=/home/gsc/.cache/glassmind/continuous-20261010/syntaxnorm-run.
Batch started2026-10-11T10:29:34Z; native work completed by10:32:37Z;
archive/rollback completed10:33:03Z. No jobs remain from this batch.
No commits, new agents, LLM/authentication calls, services, downloads or retraining.
Launcher controls required model/login selection; worker did not inspect credentials,
read API keys or initiate another model session.

## Exact candidate and preregistration

PLAN.md and24 authored cases were hashed in pre-edit.json before implementation.
The previous unnormalized width4 candidate (LAS17879,UAS19051) stayed rejected
and was not rerun. Its bounded expansion mechanics were reused, not its claims.

Only the search ranking objective changed: sum of per-state legal-action
log-softmax values, fixed temperature1.0. Each local value is
(score-max_legal)-ln(sum_legal exp(score-max_legal)), after converting existing
f32 scores to f64. Every input score, including illegal actions, is checked finite;
normalized values and cumulative totals are also checked finite. This deterministic
ranking objective is not calibrated probability or semantic proof.

Width4, stable retained-parent/ascending-action ties, unchanged labels/tags/features/
legal states/weights, no state merging, exactly2*n steps (n<=128). At most4 parents,
4 descriptors and4 selected successor states; <=129 raw scores and129 legal-action
pairs for one parent, <=132096 scored actions and1024 selected clones per sentence.
BOUND-NOTE.md corrects PLAN's singular-vector wording before DEV; objective unchanged.
Same16KiB input,128KiB JSONL rows and128MiB model limits. Width1 delegates unchanged
greedy for compatibility only; all widths other than1/4 refuse.

Rejected API adds infer_normalized_beam/predict_normalized_beam; explicit standalone
predict --normalized-beam selects4. Successes and row refusals identify
search_mode=normalized_beam_4_logsoftmax_t1_stable. Width1 metadata identifies
normalized_beam_1_greedy_compat. Default search_mode is absent from serialized JSON.
No new mode survives rollback. Candidate syntax.rs43549 bytes (+9381), standalone
6134 (+686); binary12398512 (+209336, includes debug info).

## Measured gates and regressions

Unchanged tools/reading/syntax/parent/audit.py grades all integer words, including
punctuation. No exclusions or changed evaluation definitions. Supplied baseline
outputs were hash-verified and reused; no baseline full inference was repeated.
Source/binary/model/probes/adapter/scorer/patch frozen before first100 then full2001,
each candidate run once. No post-DEV search/training edits or tuning.

| Group | Sentences | Words | LAS baseline -> candidate | UAS baseline -> candidate |
|---|---:|---:|---:|---:|
| Fixed first100 | 100 | 2319 | 1738 ->1738 | 1834 ->1833 |
| Full DEV | 2001 | 25148 | 19423 ->19428 | 20540 ->20547 |
| Nonduplicate forms | 1933 | 25011 | 19286 ->19291 | 20403 ->20410 |
| TRAIN-duplicate forms | 68 | 137 | 137 ->137 | 137 ->137 |

All2001 outputs are valid single-root trees, zero refusals. Root correctness
1694->1693; exact labeled sentences828->828 with all828 retained. Authored probes
24cases/72coreedges:72->72 correct, zero losses. Parent independent probes were
not read or run here; independent acceptance is NOT claimed. Local LAS failure
already rejects the candidate, regardless of further parent results.

| Full edge transitions | Correct -> correct | Correct -> wrong | Wrong -> correct | Wrong -> wrong |
|---|---:|---:|---:|---:|
| LAS | 19411 | 12 | 17 | 5708 |
| UAS | 20527 | 13 | 20 | 4588 |

All62 changed edges occur in16 previously inexact nonduplicate sentences.
All25148 edge transitions and2001 sentence transitions, including unchanged ones,
are in candidate/edge-transitions.jsonl and sentence-transitions.jsonl, also archived
under rejected/candidate/. The62 changed edges are copied into reports/changed-edges.json.
Full group transitions and unchanged scorer output are in reports/summary.json and
reports/{baseline,beam}-score.json. Duplicate cases remain included in every full gate.

The score-offset correction avoids the earlier catastrophic result, but its tiny
net change does not meet the frozen practical-improvement criterion. No score
promotion: this second search objective is rejected. Future work must change
training or features, not attempt a third score tweak/temperature/width sweep.

## Validation, timing and preservation

All15 scoped reading library tests passed, including3 new search/objective tests
and existing40 raw-tokenization authored contrasts. New checks cover additive
score-offset invariance, legal-only denominator, numerical extremes/nonfinite
inputs, exact ties, deterministic bounded trees, invalid widths and128/129 bounds,
width1 equivalence, and identical raw spans/forms/tags. Existing model serialization
and invalid-input tests passed. Reading all-target clippy -D warnings, package fmt
and standalone release build passed. No failed build/test/evaluation attempts.

Unmodified16 input/span contracts passed, plus actual-model NaN rejection. Baseline
and candidate default contract outputs match byte-for-byte; normalized refusals
match after removing explicit mode metadata. Original and new actual-model probes
passed byte roundtrip, width1 compatibility, deterministic valid trees at1/2/3/127/128,
invalid widths and129 refusal. Full default forms and raw ran once each and match:

- forms059e5ca85b312f9ea692d01fcb30e67f6334fad002b704e686c59ecb7638710d
- raw4c7b1b65161ebb82bd4c5f1bb7a18646dba2b2d6c613f5986f6c0cd51dbe676a

Single foreground process measurements, including model load: normalized full
0.470237s/60348KiB peakRSS; fixed100 0.066706s/60416KiB; candidate default full forms
0.156179s/60304KiB, raw0.176575s/60576KiB. These short single-run times are noisy;
no steady-state throughput claim. Timing command/rusage files remain external.
Release build77s. Every cargo invocation set TMPDIR outside /tmp and used the
required systemd8GiB/0swap scope, setsid --wait, timeout600s/kill-after10s,
-j1 except fmt, tests --test-threads4. Existing assigned CARGO_TARGET_DIR used.
Outer scope verified8GiB/0swap; observed outer peak550522880bytes excludes sibling
cargo scopes, whose peak memory was not separately recorded. Native RSS is measured.

Before rollback73 archive files/61209688bytes were copied and each hash verified.
ARCHIVE.json pins rejected/manifest.json SHA256
10855cb6578931071c974c0de898066e1ab3d933154e6736fa639984e6fbf0e2.
FREEZE.json pins pre-DEV inputs/source/patch/binary; ARTIFACTS.json pins runtime
outputs/baseline files; IDENTITIES.json gives exact candidate source/binary/model
hashes and sizes. After rollback pre-edit runtime hashes match and git diff is empty
for both owned files; rejected.patch applies cleanly. No redundant tests after an
exact rollback. No main integration or workspace-wide suite was needed for rejection.

## Source/license and reproduction

No newly downloaded data. Reused EWT r2.18, revision
b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c, CC-BY-SA-4.0:
TRAIN15034493bytes + DEV1898574bytes =16933067bytes. NOTICE.md gives source URL and
hashes. New authored probes24cases/2872bytes, MIT, hash/version in pre-edit.json.
Official TEST, held-out items, hidden generators and parent contrast files were
not accessed. Runtime sees only forms/text, never gold annotations. The scorer and
reporting adapter read gold DEV for grading and TRAIN for form-duplicate groups.

Fixed model12554724bytes SHA256
47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6.
Candidate binary b15f1a0a548dea766fa9a360dd834e0060926866082e98272d7f640052b3d84a.
Candidate full output e008c7dd95ae8a06835a5afc2bc7688ba36bf10435d42f97a7ddab6108a338c2.
Rejected patch c321e817790948ed6ab019bd64c250b256eeb0011caac7b3fcad220b77eec9a3.

For reproduction only, use a disposable matching source checkout and a fresh RUN
containing the hash-identical baseline/ bundle. Do not deploy this rejected patch.
reproduce.sh contains all exact capped cargo and model-probe invocations; evaluate.py
refuses to overwrite a DEV freeze. Preserve the assigned existing target directory.

```sh
git apply --check tools/reading/syntax-normalized/rejected.patch
git apply tools/reading/syntax-normalized/rejected.patch
RUN=/path/to/fresh-replay \
SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source \
CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/syntaxnorm-run/target \
PYTHONDONTWRITEBYTECODE=1 tools/reading/syntax-normalized/reproduce.sh
```

Read-only rescoring of retained outputs, without repeating inference:

```sh
python3 tools/reading/syntax/parent/audit.py \
  --source /home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source \
  --predictions /home/gsc/.cache/glassmind/continuous-20261010/syntaxnorm-run/candidate/beam-predictions.jsonl \
  --out /path/to/fresh-audit
```

This is rejected experiment evidence, not an accepted runtime improvement.
Independent parent linguistic checks remain unperformed; authored probes and
DEV cannot establish broad semantic understanding. No claims authorize answers,
memory changes or coding actions.
