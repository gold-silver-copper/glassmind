# Word trigram: complete, rejected, runtime restored

One bounded prospectively specified experiment, no post-sample/full tuning.
**734 good / 603 bad / 3 ties out of 1340** versus frozen byte6
**707 / 633 / 0**: +27 good, below the fixed **>=747** acceptance criterion.
The candidate is REJECTED. No accepted runtime improvement is claimed. The
original new Rust sources and patch were archived and hash-verified before
removing both runtime files. Only `tools/reading/word-grammar/` is left changed.
No commits, agents, LLM calls, services, background jobs, credential reads,
API keys, held-out items, new official BLiMP items or hidden evaluation access.
Host model/auth selection was inherited, not independently attested or changed.

## Exact change and frozen experiment

Base commit `b569ac0eb0bab5a8b9d39cda904c816e77c08e63`. PLAN.md was written before
source edits/training/dev. The frozen original source is under `rejected/source/`:

- `crates/reading/src/word_probability.rs`: fixed Unicode tokenizer, TRAIN-only
  minimum-count-two vocabulary, word trigram counts, recursive Witten–Bell
  interpolation with positive 0.5 unigram prior, deterministic bounded binary
  persistence, text-only cost API and three focused tests.
- `crates/reading/src/bin/reading-word-probability.rs`: separate train/score/evaluate
  adapter; includes the unchanged original byte scorer and saves both costs and
  choices for every original row. IDs/groups are adapter metadata only.
- New ownership-local plan, provenance verification/freezing, reproduction and
  offline cost-audit scripts, archived source/patch, full reports and handoff.

No lib exports, manifest/dependency/lockfile edits, existing scorer changes,
main CLI/QA/loop changes, README edits or other-track changes. `git diff --exit-code
-- crates/reading Cargo.lock` passes after rollback. The archived patch passes
`git apply --check`; it is for inspection/reproduction, not deployment.

Segmentation is maximal original Unicode alphanumeric runs, followed by scalar
lowercasing; Unicode whitespace is discarded, each other scalar is its own token.
No Unicode normalization; apostrophes, hyphens and combining marks are separate.
All six TRAIN members in lexical order determine vocabulary/counts, in two passes.
Every LF-delimited line is a document, with the baseline's LF/CR removal; two BOS,
one EOS. Vocabulary sorted lexicographically; explicit EOS/UNK/BOS IDs, BOS never
predicted. Unknown spellings collapse to UNK. The score is total negative log2
TOKEN probability including EOS, no length normalization or byte mixture; lower
wins, absolute difference <=1e-9 ties. No generation API or runtime model calls.

## Evaluation, regressions and decision

Sources/model/binary and inputs were frozen in `reports/frozen.json` before dev.
Exactly one first-20 sample, then exactly one full 1340 candidate evaluation;
no calibration, alternate settings, retraining, post-full changes or additional
candidate inference. No pipeline failure or corrected completed report occurred.

| Scope | Byte6 good/bad/tie | Word3 good/bad/tie |
| --- | ---: | ---: |
| Fixed first 20 rows | 12 / 8 / 0 | 11 / 7 / 2 |
| Full 1340 rows | 707 / 633 / 0 | 734 / 603 / 3 |

Full before/after transitions, rows byte6 and columns word3:

| From / to | Good | Bad | Tie |
| --- | ---: | ---: | ---: |
| Good | 551 | **154** | **2** |
| Bad | 183 | 449 | 1 |
| Tie | 0 | 0 | 0 |

All **156 baseline regressions** are listed with original row number/ID/group,
input-row SHA256 and all four costs in `reports/full-report.json` under
`baseline_regressions`. Complete costs and choices for all 1340 original rows are
in `reports/full-rows.jsonl`. All 67 group totals and transitions are in the
report; `GROUPS.md` displays all totals. 30 groups improve, 25 regress, 12 equal.
Every baseline group and the full denominator exactly reproduce the retained
original report, using the unchanged original Rust byte scorer and byte6.bin.
The sample is a pipeline check, not a representative performance estimate.

This sole model experiment is rejected at its original gate. No other variants
were attempted or rejected. The positive net change does not override the
prospective threshold, and individual prior correct preferences are not preserved.

## Artifacts, resources and validation

Raw run directory (R):
`/home/gsc/.cache/glassmind/continuous-20261010/wordgrammar-run/experiment`.

- `R/word3.bin`: **125,312,749 bytes** (<150 MiB), SHA256
  `5a28d340062003fb1962d931b595abd10cf049b3cd762700be973beae6ed14f5`.
- `R/reading-word-probability`: **11,235,344 bytes**, SHA256
  `8d354c78f6e8bde1518d6ead517275230166d90812c24a69f885eff6ac8353ee`.
- `R/frozen.json`: identities of source, plan, scripts, manifest/lockfile,
  model, binary and inputs, recorded before sample/full.
- `R/{sample,full}-costs.jsonl`: original raw adapter output, never replaced.
- `R/{sample,full}/{rows.jsonl,report.json}`: original audited results.
- `R/train.json`, `R/*-resources.json`, `R/{build,clippy,fmt}.log`, `R/rustc.txt`.
  Reports copied under `tools/reading/word-grammar/reports/`; frozen source and
  original patch/hash manifest under `rejected/`.

Training: **1,179,014 documents, 14,134,935 tokens, 69,156 lexical vocabulary
entries, 6,227,234 ngram entries**. 5.055 seconds, peak child RSS 366,268 KiB.
Full evaluation: 1.813 seconds, peak RSS 512,680 KiB. Release build: 64.596 seconds,
510,192 KiB. Raw measurements include exact commands and exit codes.

Passed: three meaningful candidate tests (normalization over predicted vocabulary
with seen/unseen histories; empty/OOV/Unicode/punctuation behavior; deterministic
serialization across independent training plus score roundtrip and malformed/
truncated/trailing model rejection), and two unchanged original byte tests.
Five tests total, no giant suite. Package fmt check, affected binary/tests Clippy
with `-D warnings`, release build, shell/Python syntax checks, all input/source/
artifact hash checks and 67-group original baseline replay passed.
`validation.json` records the original test invocation/results; test stdout was
returned by tool session 52837, not retained in a separate log. No duplicate test
run is claimed. No full workspace or unrelated-track tests were needed.

Every cargo invocation set the inherited non-/tmp TMPDIR, ran in
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, used `-j1` except fmt, and tests used `--test-threads 4`.
Training/evaluation had the same bounds. Outer scope independently verified at
8 GiB/zero swap. Existing target directory reused. All owned jobs completed.

## Reproduction and offline audit

Offline cost replay requires no model execution and can be run in the restored
checkout, with a NEW output directory (existing reports are never overwritten):

```bash
R=/home/gsc/.cache/glassmind/continuous-20261010/wordgrammar-run/experiment
python3 tools/reading/word-grammar/audit.py \
  --input /home/gsc/.cache/glassmind/grow-fast-20261010/reading-data/grammar.jsonl \
  --scores "$R/full-costs.jsonl" \
  --baseline-report /home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability/grammar-full.json \
  --output "$R/coordinator-offline-replay" --full
```

To rebuild/retrain/re-evaluate the rejected experiment, apply its archived patch
in a disposable checkout at the base commit with this evidence directory copied
in. Do not integrate it into production. The following reproduces fixed settings,
input verification, build checks, freeze, first20 then full; use fresh WORD_RUN:

```bash
git apply tools/reading/word-grammar/rejected/candidate.patch
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/wordgrammar-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/wordgrammar-run/target
export WORD_RUN=/home/gsc/.cache/glassmind/continuous-20261010/wordgrammar-run/coordinator-replay
bash tools/reading/word-grammar/reproduce.sh
```

This consolidated script was syntax-checked; this worker executed its steps
individually, not a second consolidated run. Original compile/train/eval commands
are also in resource reports. The original byte model is only opened read-only;
this script never invokes the byte trainer or overwrites byte6.bin.

## Data, limitations and final decision

**0 newly downloaded bytes.** Reused mounted BabyLM2025 strict-small TRAIN:
18,139,949-byte archive, 54,351,399 raw extracted bytes across six members, README
3,589 bytes. Revision pinned by archive SHA256
`6d9d2cb3b3feccf0669e74173edb55de6c7f12412ee02935d16b1b6b44160b39`;
no upstream commit supplied. `DATA.md` and `verified-inputs.json` give all member
sizes/hashes, dev identities and mixed-source licensing: BNC conditions, CHILDES
noncommercial terms, Gutenberg terms, qualified Wikimedia terms, unresolved
blanket redistribution grants for OpenSubtitles/SwDA. No model redistribution
license is inferred. Corpus/model stay outside git in the run cache.

This measures grammatical pair ranking, not semantic understanding or truth.
Lowercasing and tokenization discard case/spacing and are noninvertible; UNK
collapses distinct OOV spellings. Local trigrams miss longer dependencies, and
total cost can favour shorter strings. These token costs are NOT byte likelihood,
reading bits/byte or comparable byte compression. Dev was previously exposed;
no held-out generalization claim. Rust Unicode tables/platform floating-point
behavior are toolchain-dependent; the compiler identity is retained.

Frozen byte path/source/artifact remain unchanged: byte6 SHA256
`fb187a60997ea56bb9433f1e9ccd9ab81c571d519e648523750277b865999c93`, 68,861,296 bytes
at `/home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability/byte6.bin`.
Historical reading120doc BPB **2.1892595993** and disclosed **33/120** TRAIN overlap
are intact, not reevaluated or relabeled.

Final decision: **REJECTED, 734/603/3 vs 707/633/0, threshold 747 unmet**.
Runtime fully restored; original source/patch/binary/model/results retained for
coordinator audit. One completed batch, no unfinished work or surviving jobs.
