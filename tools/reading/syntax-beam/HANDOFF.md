# REJECTED — fixed width-4 cumulative-score syntax beam

One bounded experiment completed; do not integrate the rejected runtime patch.
Both runtime files were restored byte-for-byte after archiving and verifying the
candidate source, binary and reports. Only tools/reading/syntax-beam is uncommitted.
No main command, shared manifest, scorer, tokenizer, model, historical report or
prompt was changed. No commits, agents, LLM calls, downloads or services were made.
All owned foreground jobs have completed. Launcher owns Astra fast/ChatGPT auth;
this worker did not inspect credentials, read keys or initiate authentication calls.

Base source: 86b1e545efbeaa9262d157125fccd9841ada7794.
External run: /home/gsc/.cache/glassmind/continuous-20261010/syntaxbeam-run.
The batch began around2026-10-11T10:16:55Z and completed around10:24Z.

## Frozen hypothesis and exact implementation

PLAN.md and24 authored role/order/question/punctuation contrasts were hashed before
runtime edits (pre-edit.json). One candidate: retain width4 alternatives using an
unnormalized f64 sum of the existing finite f32 transition scores. Enumeration is
retained-parent order then ascending action ID; exact score ties preserve that order.
No state merging or normalization. Tags/features/legal actions/root/cycle checks are
unchanged. Model version, save/load format and weights are unchanged.

The rejected patch adds Model::infer_beam and predict_beam, a standalone predict
--beam switch, and optional search_mode omitted from default JSON. Explicit beam
successes and JSON row refusals identify beam_4_sum_stable. Width1 delegates to
unchanged greedy inference and identifies beam_1_sum_stable; all widths other than
1 or4 refuse. Beam1 was compatibility-only, not a second evaluated candidate.
Default infer/predict keep their original transition path and output.

Input bounds remain128 tokens,16KiB text/forms and128KiB JSONL rows. Search performs
exactly2*n steps, at most256. Existing64-label cap gives at most129 action scores
per state, at most132096 examined scores and1024 selected successor clones. Four
retained parents, at most four descriptors, at most four temporary successors
while replacing the parents; no materialized pool of expansion states. All transition
scores are checked finite, including illegal actions; nonfinite totals refuse.
Existing loaded-model weight bounds make cumulative overflow unreachable in normal
models. PLAN's conservative128-label figure was corrected in BOUND-NOTE before DEV.

## Measured result and gates

Unchanged independent tools/reading/syntax/parent/audit.py grades all integer words,
including punctuation. No DEV exclusions or evaluation-definition changes.

| Run | Sentences | Words | LAS baseline -> beam | UAS baseline -> beam |
|---|---:|---:|---:|---:|
| Fixed first100 | 100 | 2319 | 1738 ->1586 | 1834 ->1700 |
| Full DEV | 2001 | 25148 | 19423 ->17879 | 20540 ->19051 |
| Nonduplicate TRAIN forms | 1933 | 25011 | 19286 ->17742 | 20403 ->18914 |
| Duplicate TRAIN forms | 68 | 137 | 137 ->137 | 137 ->137 |

LAS delta-1544 fails required>=19673 (+250); UAS delta-1489 fails required>=20540.
All2001 outputs remain valid single-root trees; zero refusals. Root correctness
1694->1511. Full exact labeled trees828->661. The duplicate group is unchanged.

Every correctness transition is retained in external edge-transitions.jsonl
(5276 changed edges) and sentence-transitions.jsonl (all2001 sentences;1145 changed).
Both are under candidate/ and rejected/candidate/. ARTIFACTS.json pins all hashes.

| Metric | Correct -> correct | Correct -> wrong | Wrong -> correct | Wrong -> wrong |
|---|---:|---:|---:|---:|
| LAS edges | 16892 | 2531 | 987 | 4738 |
| UAS edges | 17971 | 2569 | 1080 | 3528 |
| Exact labeled sentences | 617 | 211 | 44 | 1129 |

All regressions and gains are in the nonduplicate group. Its retained correct
LAS/UAS counts are16755/17834;549 exact sentences remain correct. All68 duplicate
sentences and137 duplicate edges remain correct. Full group ledgers and original
independent scores are preserved, including the scorer's secondary punctuation view.

Authored contrasts:60->54 of72 correct core edges; six previously correct edges
lost, zero gained. All-three-core-edge cases15->14 of24. Failed preservation gate.
Three fronted-adverb cases lose helped's root to Yesterday/appos; two parenthetical
cases turn the object into the root; one passive changes obl:agent to obl.
Exact cases are reports/authored-losses.json; full native predictions remain external.
These are worker-authored tests, not independent parent verification. Parent's
independent contrast supplement was not run here; local rejection already suffices.
No predictions authorize answers, memory or code edits. Scores are neither
probabilities nor semantic proof. The result rejects this search objective with
these weights; it does not establish that every form of beam search is unsuitable.

## Validation and preservation

Passed: scoped15 reading library tests (including3 new beam tests and all40 existing
raw contrasts), standalone release build, reading all-target clippy with-D warnings,
package fmt check;16 original adapter contracts plus actual-model NaN rejection;
original model-check probe and new actual-model beam bounds/compatibility probe.
New tests cover ties, deterministic search, width1 equivalence, invalid widths,
128/129-token bounds, raw exact spans, nonfinite scores, and a constructed case where
a lower-scoring prefix recovers a better cumulative path. Model byte roundtrip passes.

Full default forms output remains SHA256
059e5ca85b312f9ea692d01fcb30e67f6334fad002b704e686c59ecb7638710d.
Full default raw output remains SHA256
4c7b1b65161ebb82bd4c5f1bb7a18646dba2b2d6c613f5986f6c0cd51dbe676a.
Both are byte-compared over2001 rows against the approved baseline. Default JSON
refusal contracts remain unchanged; all16 baseline/candidate contract outputs are
byte-identical. Beam refusals differ only by search metadata. Final archive/source
reverification is in reports/final-verification.json.
Main binary integration/workspace checks were not performed; rejection needs none.
After rollback, runtime hashes equal pre-edit hashes; tests were not redundantly rerun.

Initial clippy rejected JSON indexing in the new refusal-metadata code; fixed by
constructing the complete JSON object before DEV, then all checks passed. One harness
attempt stopped before any candidate DEV predictions because /usr/bin/time was
absent. predev-measure-failure/ preserves it and its frozen adapter. The final harness
uses unchanged tools/reading/syntax/measure.py in a fresh Python process per run.
Final source/binary/adapter/model/probes were frozen before candidate DEV (FREEZE.json).
The fixed first100 and full2001 were then each evaluated once. No post-DEV tuning,
second search candidate, width/temperature/features sweep or retraining occurred.

Single foreground full-form runs, including process/model-load time: baseline
0.181322s, peak59768KiB; beam0.453426s, peak60032KiB; slowdown2.50066x. Candidate
default0.229646s, peak60148KiB. Short single runs have timing noise; these are not
steady-state parser throughput estimates. Exact timing commands/rusage are external
and summarized in reports/summary.json. Model remains12554724 bytes. Standalone
release binary12189176->12382896 bytes (+193720, includes debug info); candidate
syntax.rs34168->41808 bytes, standalone source5448->6004 bytes, all now rolled back.
Outer scope verified MemoryMax8589934592,MemorySwapMax0. Every cargo invocation set
TMPDIR outside /tmp, used the required8GiB/0swap systemd scope, setsid --wait and
600s timeout/10s kill grace; -j1 except fmt, tests --test-threads4.

## Source, identities, archive and reproduction

Reused EWT r2.18, revision b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c,
CC-BY-SA-4.0: TRAIN15034493 bytes, DEV1898574 bytes; total16933067. New downloads0.
New authored contrasts24 cases/3381 bytes, MIT. NOTICE.md gives source URL and hashes.
No official TEST, held-out items, hidden generators or old private logs were read.

Fixed model SHA256:47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6.
Candidate binary:04b75f7bcf10d14f808db97013beeb2fc2e91d9d9b5ca69846e991a746ba6b3f.
Candidate full predictions:411b002debad6b1575885d3d2c9f1ed23f673118d9e8bf76ca4196b5588a2737.
Candidate syntax.rs:d045f9a83232c78995c5e466e7843b759f9372244b20e93152dc031eff3cf03a.
Candidate standalone source:fc9d33f55e3d3cfe574dff1bb9091edb87f3294e6c5b27be91957da8d81181f5.
Rejected patch:bf1e1ef1901d674cd0d30914c5824db7d153abbecb15dd1c8934cbc5e0ed1518.
Other exact identities/bytes: IDENTITIES.json, FREEZE.json and ARTIFACTS.json.

Before rollback, rejected/ archived57 entries/58265881 bytes, verified every entry.
Its manifest SHA256 is943429fc4ad6c14be4f6c9817a4201c038991e962a71190bb94fcc29f8def863.
ARCHIVE.json identifies its absolute path. Raw predictions and ledgers stay outside
Git; small summaries, rejected patch and reproduction tools remain in this directory.

For reproduction only in a disposable matching checkout, apply rejected.patch;
never integrate it into the main runtime. Prepare a fresh RUN with the unchanged
approved baseline/ bundle, and preserve the assigned existing CARGO_TARGET_DIR.
The frozen script will refuse to overwrite an existing candidate DEV freeze:

```sh
git apply --check tools/reading/syntax-beam/rejected.patch
git apply tools/reading/syntax-beam/rejected.patch
RUN=/path/to/fresh-replay \
SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source \
CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/syntaxbeam-run/target \
PYTHONDONTWRITEBYTECODE=1 tools/reading/syntax-beam/reproduce.sh
```

reproduce.sh contains the exact capped fmt/test/clippy/release and model-probe
commands. evaluate.py records native invocation arguments and calls the unchanged
independent scorer. Read-only rescoring of retained outputs needs no new inference:

```sh
python3 tools/reading/syntax/parent/audit.py \
  --source /home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source \
  --predictions /home/gsc/.cache/glassmind/continuous-20261010/syntaxbeam-run/candidate/beam-predictions.jsonl \
  --out /path/to/fresh-audit
```

No improvement is claimed. The quantitative and role-preservation gates failed;
only rejected experiment evidence is handed back for coordinator integration.
