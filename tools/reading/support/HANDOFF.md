# Reading support batch — REJECTED, evidence only

Coordinator review independently recomputed both calibration frontiers from
the retained model, features and official exactness labels: neither reaches
90% precision with at least 30 accepted predictions. It also verified 75
provenance hashes and byte equality of the complete 10,570-row v1 replay.
[Parent review](reports/parent-review.json) records these checks. No second
training or full inference run is claimed. The rejected patch remains unapplied.
The original 17 evidence files (505,808 bytes) are archived and hash-verified
under `/home/gsc/.cache/glassmind/continuous-20261010/qasupport-run/submitted`;
the completed worker's worktree and temporary login link have been removed.
The following handoff describes the original worker submission.

One bounded experiment on base `1003add43fe9e1970ee0a5380fc15c7304f94aa1`.
No commits. **No active runtime changes remain. Do not integrate the rejected
adapter or model.** Only new `tools/reading/support/` files remain untracked.
`crates/reading`, the scorer, CLI, manifests, historical prompts and README files
are unchanged. No agents, LLM calls, credentials, downloads or held-out data were
used. All owned build/training/inference processes finished.

## Hypothesis and design

[PREREGISTER.md](PREREGISTER.md) was written before implementation or dev access.
The TRAIN audit examined the first 16 deterministically selected unused TRAIN
rows and question-kind counts. It motivated question/answer kind, clause anchor
coverage and ordering features, with no predicate/entity/answer lookup tables.

The rejected patch adds **only** a separate Rust `reading-support` binary:
48 bounded features, optional version-1 support model, finite tree/score checks,
exact original UTF-8 proposal/evidence offsets. A selector can clear an already
selected answer; it cannot revive a refusal or change a selected answer.
The frozen ranker and its model serialization are untouched. A Python offline
adapter trains 60 squared-error depth-two trees, learning rate .1, minimum leaf
15, 15 FIT-only quantile cutpoints per feature, on old-selected predictions.
No hyperparameter search or alternate feature experiment occurred.

TRAIN selection excludes all original 64 fit +16 calibration articles by their
frozen hashes. Next96 article hashes yield 8000 support-fit rows; next32 yield
2000 support-cal rows, each capped using ascending SHA256(questionID).
[selection.json](reports/selection.json) contains every selected article hash and
question ID. Runtime receives only context and question. TRAIN gold exactness
labels never enter inference inputs. Fit selected 1260 rows, 581 exact; calibration
selected 304, 140 exact. Article splits are disjoint; the selector was not fitted
on ranker fit/calibration articles.

Both richer and margin-only selectors used the **same frozen** maximum-coverage
threshold rule: >=90% observed calibration exact precision, >=30 accepts, ties
included together. Neither had a qualifying threshold. Neither is calibrated as
a probability. No lower threshold was selected or tested on dev.

## Results and rejection

[train-audit.json](reports/train-audit.json) records the failed gate. Post-freeze
TRAIN-only diagnostics show the best qualifying-size prefix was 24/31 (77.42%)
for the richer score and 28/45 (62.22%) for margin. These are diagnostic maxima,
not replacement thresholds or evidence of success at the target precision.

| Full dev, all 10570 | Frozen v1 | Rejected support | Margin-only |
|---|---:|---:|---:|
| Raw exact correct | 2532 | 2532 | 2532 |
| Raw EM / F1 % | 23.954588 / 34.518425 | unchanged | unchanged |
| Selected EM / F1 % | 7.710501 / 8.700190 | 0 / 0 | 0 / 0 |
| Selected right / wrong | 815 / 791 | 0 / 0 | 0 / 0 |
| Refused | 8964 | 10570 | 10570 |
| Selected exact precision | 50.747198% | undefined | undefined |
| Coverage | 15.193945% | 0% | 0% |

The null-threshold full run is a fail-closed diagnostic, **not a candidate for
shipping**. It loses all 815 correct selections and suppresses 791 wrong ones.
Every old refusal is preserved; no new claims, changed proposals, evidence,
rank scores or margins. All 1606 individual transitions (IDs, original answers,
correct/wrong-to-refused labels) are in
`../qasupport-run/support/full.transitions.json`, SHA256
`a7e32e846fcf7138b929a44dfe61a44aca4e5a26dc871b04d461c15f5ad363f2`.
Large inputs/outputs stay outside git in that run directory.

The fixed SHA256(ID) first200 was evaluated before full dev: v1 16 right,
17 wrong, 167 refused; both rejected selectors 0/0/200. Raw EM23%,
F1 31.2087565 unchanged; selected v1 EM8%, F1 8.8444444. Full reports are
[sample.json](reports/sample.json) and [full.json](reports/full.json).
Acceptance (>=400 correct, >=80% precision, more correct than margin at
precision within .02) **fails**. No meaningful dev improvement is claimed.

## Independent contrasts: substantive failure

[probes.json](reports/probes.json) records all 15 authored probes, expected
behavior, v1 output, rejected output, unthresholded classifier score and features.
They are diagnostic constructions, not an external benchmark. Seven negative
cases refuse; all eight positive cases fail through refusal. The 7/15 total is
vacuous abstention, not semantic accuracy.

Most decisively, the classifier gives the **same score, 0.867702 rounded**, to:

- “Nora painted the bridge in 2008.” / “When did Nora demolish the bridge?”
- “Nora demolished the bridge in 2008.” / the same question.
- “Nora never demolished the bridge in 2008.” / the same question.

All three v1 proposals are `2008`; only the middle source supports the requested
relation. The ensemble never split on the negation feature. For “Who thanked
Felix?”, changing “Nora thanked Felix” to “Felix thanked Nora” raises the score
of the wrong `Nora` proposal from .411348 to .614031. Another reversed hire-role
pair similarly raises the wrong score. Source occurrence and answer shape do
not supply predicate/argument support. Person/place/date, numeric-vs-person/place,
object role, Unicode and positive counterparts are all retained in the report,
including inherited v1 refusals. No logical verification claim is made.

## Validation and artifacts

- Release build passed. Six existing QA +two existing byte probability tests
  passed; two experimental feature/UTF-8/model-bound tests passed.
- Scoped crate formatting passed. No workspace suite, main CLI or metadata
  replay was run; those sources are unchanged and remain coordinator-owned.
- Independent `reading-qa` v1 replay is byte-identical to the frozen complete
  10570-row prediction file, SHA256
  `37db70ae0c949a89427a707c37812a666d9507a2192bcb75ed3e4145d3a598a7`.
- Unmodified `tools/reading/qa/run.py` imports the pinned official scorer
  function ASTs with `SQUAD_SOURCE`; no denominator/sentence dropping or scorer
  changes. ID-set assertions and selected-to-refused invariants passed.
- `git apply --check tools/reading/support/rejected.patch` passed after removing
  the experimental source. Tracked runtime diff is empty.

[provenance.json](reports/provenance.json) records source, model, binary, scorer,
input, selected TRAIN labels and result byte sizes and SHA256 hashes. The removed
experimental source hash describes the exact source in the preserved patch.
Key identities:

| Artifact | Bytes | SHA256 |
|---|---:|---|
| Frozen v1 model | 873881 | `b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5` |
| Rejected support model | 28675 | `38eed9ccb1b8fc0d6b6e76471b47efd0a2e16253ab7543d810ae27766716b13e` |
| Experimental Rust source | 11524 | `ea820035a08a4f483cdaa794860012480c5999792fc6e8bf297b407ca7b9bb01` |
| reading-support binary | 11808384 | `7a9bdb4a0a3c1a8655c019d6d59e58929814fb1c277075972fcd36810eeae3b2` |
| reading-qa binary | 11343480 | `c1c0a664e37fb7895735a044f3c906486c961472dc58e533f2c30e5a51931ffd` |

Run directory: `/home/gsc/.cache/glassmind/continuous-20261010/qasupport-run/support`.
Logs: `build.log`, `fmt.log`, `tests.log`, `train.log`, `sample.log`, `full.log`,
`probes.log`, `audit.log`. The raw fit/cal feature predictions, labels, trained
model, complete calibration frontier and per-row TRAIN diagnostics are retained.
[features.json](features.json) names every feature in vector order;
[diagnostic.json](reports/diagnostic.json) exposes every used split feature.

## Source and license

No new external data or dependencies. Reused readonly public SQuAD1.1 TRAIN/DEV
at revision `eee5fdbf62f8613a7812b03419e6b29617b74fd1`, dataset CC-BY-SA-4.0,
repository/scorer MIT. Attribution/source:
https://github.com/rajpurkar/SQuAD-explorer/tree/eee5fdbf62f8613a7812b03419e6b29617b74fd1
and https://rajpurkar.github.io/SQuAD-explorer/ .
TRAIN 30,288,272 bytes (87599 questions); DEV 4,854,279 bytes (10570), total
35,142,551 dataset bytes. Their exact hashes and license/provenance hashes are
in provenance.json. New derived fit context/question JSONL is 6,816,473 bytes;
cal input is 1,694,806 bytes; offline label JSON files are 8,096,389 and 2,016,644
bytes. Selection report is 349,639 bytes. Derived examples retain the source
license. The 15 tiny contrast cases are original diagnostic text.

## Reproduction

From this worktree, deliberately restore the rejected experimental source for
reproduction only, using the existing target directory and disk-backed TMPDIR:

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/qasupport-run/target
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/qasupport-run/tmp
export SQUAD_SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/readingqa-source
export SUPPORT_RUN=/home/gsc/.cache/glassmind/continuous-20261010/qasupport-run/support-replay
git apply tools/reading/support/rejected.patch
bash tools/reading/support/reproduce.sh
```

The script executes prepare, release build, focused tests, TRAIN fit/calibration,
fixed sample, full dev +v1 byte replay, independent probes, then diagnostics.
It uses `TMPDIR=... systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0
setsid --wait timeout --kill-after=10s 600` for every workload, `cargo -j1`, and
`--test-threads 4`. Actual commands were run individually with these caps; the
whole convenience script was not re-run. Outer scope also verified 8GiB/zero swap.
Keep the patch unapplied in integrated production source.

## Limits

This is a measured negative result, not a solution to relation verification.
Only 1260 selected fit examples trained the classifier; SQuAD has no genuine
unanswerables. Exactness labels conflate boundaries, answer kinds and support.
The lexical stemmer and anchor ordering do not parse predicate/argument roles.
The training signal failed to make negation or relation contrasts operative.
No relation features or cutoff were retuned after dev. Repeating this candidate
with a lower cutoff would not repair the demonstrated semantic failure.
