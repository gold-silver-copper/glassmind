# Reading probability scorer: bounded worker handoff

**Complete, uncommitted batch.** Base source
`ed81553d1d2be8b1dbe2df6228bd5f560d510170`; evaluated implementation hashes
are in `source-hashes.json`. No commit, agents, new LLM calls, services,
API keys, credential reads, or surviving background work. The worker did
not change or independently attest its host model/auth configuration.

The preregistered hypothesis passed its two criteria, with substantial
limitations: full dev good preferences increase **578 → 707 / 1340**;
full dev byte-weighted bits per byte decrease **4.8212138605 → 2.1892595993**.
This is a classical local probability ranker, not a semantic-understanding
claim or an official held-out result. The old parse probe's
**1 good / 3 bad / 1336 ties** is a different metric and was not rerun or
reinterpreted as likelihood.

## Exact changes and boundaries

- `crates/reading/src/probability.rs`: inspectable Rust byte-count model,
  fixed smoothing, deterministic binary persistence, two focused tests.
- `crates/reading/src/bin/reading-probability.rs`: separate `train`,
  `evaluate`, and `score` commands. No generation interface or runtime model
  calls. Both orders come from identical training counts.
- `crates/reading/Cargo.toml`: sets `default-run = "glassmind-reading"` so
  adding the second binary preserves the existing package run command.
  No dependency changes; Cargo.lock unchanged.
- `tools/reading/`: preregistration, extraction/hashing, resource wrapper,
  reproduction script, post-evaluation overlap diagnostic, licensing and
  aggregate evidence. No corpus, dev strings, or weights in the repository.

Existing `crates/reading/src/main.rs` is byte-for-byte unchanged. No shared
manifest, main CLI, loop, existing scorer, README, or historical prompt was
modified. No held-out data, hidden-test code, test generator, or old private
log was read. The two explicitly requested task documents were read. Only the
allowed sememe archive and README were accessed; no sememe code/dependencies.
Web access checked primary dataset/license documentation only, no test data
or other BLiMP examples. No tuning used dev pairs, vocabulary, or groups.

## Model and normalization

Fixed before implementation/evaluation in `PREREGISTERED.md`: byte 6-gram,
no pruning, no hyperparameter selection. Alphabet is bytes 0..255 plus
EOS=256. BOS=257 is context only. Every independent document starts with
five BOS symbols; bytes then one EOS are scored. Training consumes all six
archive members in lexical order, every LF-delimited line including empty
lines; remove delimiter LF and an immediately preceding CR. A trailing LF
does not create an extra empty document. No case/Unicode/space normalization,
no vocabulary fitting, no length truncation, no cross-document contexts.
Dev strings are their raw JSON-decoded UTF-8 bytes, including any interior
newlines. The byte distribution covers invalid UTF-8 as well as valid text.

For event x and context h, N(h) is its event count and T(h) its number of
observed event types. The base distribution is
`P1(x) = (C(x) + 0.5) / (N + 257 * 0.5)`.
At each context length 1..5, shortest first:
`P(x|h) = (C(h,x) + T(h)*P(x|suffix(h))) / (N(h)+T(h))`.
An unseen context backs off unchanged. Each conditional sums to one;
positive unigram mass and interpolation keep every byte/EOS possible.
Finite contexts and positive EOS probability also make finite-document
probabilities a proper terminating distribution.

Pair preference compares total negative log2 probability including EOS;
a smaller cost wins; absolute difference <=1e-9 is a tie. This inherently
favours shorter sequences in some comparisons. Reading BPB is the sum of
all document costs (including EOS) divided by the sum of text bytes
(excluding EOS). It is **not** an average of document BPB and is not
normalized by characters/tokens. Empty documents contribute EOS cost;
a zero-byte aggregate returns null BPB. Malformed JSON rows fail loudly,
not silently disappear. All authorized dev rows were valid and retained.

The artifact format is `GMBYTE06`, followed by little-endian u64 document,
byte, and entry counts, then sorted `(packed_key, count)` u64 pairs.
Keys use a leading one and 9 bits per symbol. It stores byte subsequence
counts, not full document records; short corpus fragments are recoverable
from ngram keys. Settings are fixed by this format/code version. Load checks
magic, key ordering/symbol bounds, positive counts, truncated/trailing data,
and unigram totals; it is a research artifact reader, not an authenticated
untrusted-input format.

## Exact measurements and regressions

| Fixed dev | Unigram | Byte 6-gram |
| --- | ---: | ---: |
| Sample grammar: good / bad / ties, 20 pairs | 0 / 0 / 20 | 12 / 8 / 0 |
| Sample reading: BPB, 12 docs / 257 bytes | 4.9370112647 | 2.2666527224 |
| Full grammar: good / bad / ties, 1340 pairs | 578 / 579 / 183 | 707 / 633 / 0 |
| Full reading: BPB, 120 docs / 10399 bytes | 4.8212138605 | 2.1892595993 |
| Full reading: total bits including 120 EOS | 50135.80293528539 | 22766.110573120295 |

Samples are the preregistered first 20/12 rows, evaluated before full dev.
They are pipeline checks, not representative estimates. All 120 documents
have lower costs under the 6-gram; all six source groups have lower BPB.
All 67 grammar groups remain in `grammar-full.json`: 37 improve good count,
7 equal, **23 regress**. The largest good-count regressions include
`only_npi_licensor_present` 20→5, `wh_vs_that_with_gap_long_distance` 11→0,
and `wh_vs_that_with_gap` 10→1. No rule or setting was changed in response.

Complete pair transitions (rows unigram, columns 6-gram):

| From / to | Good | Bad | Tie |
| --- | ---: | ---: | ---: |
| Good | 393 | **185** | 0 |
| Bad | 246 | 333 | 0 |
| Tie | 68 | **115** | 0 |

There are 129 more good preferences but **54 more bad preferences** and
183 fewer ties. As a secondary, non-selection summary, assigning half credit
to ties gives 669.5/1340 → 707/1340 (49.96% → 52.76%); this is a modest grammar
result. The model is not a safe replacement for the existing parser.

**Post-evaluation overlap audit found 33/120 reading documents exactly equal
to a full training line.** No rows were removed, and no model was retrained.
This makes the existing frozen dev sample less independent than a clean
unseen-text evaluation. The audit is only exact whole-line equality; near
matches, substrings and BLiMP/train overlap were not audited. `overlap.json`
records the diagnostic without copying any text. Do not claim clean held-out
generalization or semantic understanding from the BPB improvement.

## Corpus, artifact and resources

Training version: mounted **BabyLM2025 strict-small10M**, pinned by archive
SHA-256 `6d9d2cb3b3feccf0669e74173edb55de6c7f12412ee02935d16b1b6b44160b39`,
18,139,949 compressed bytes; six members total 54,351,399 raw bytes.
All member names, individual sizes and hashes, mounted README hash, and
frozen dev hashes/denominators are in `provenance.json`. No new data downloaded.
See `LICENSES.md` for README attribution and primary-source license checks,
including CHILDES noncommercial terms and unresolved precise grants for
OpenSubtitles/SwDA. No blanket redistribution license is asserted.

One complete training pass: **1,179,014 documents, 53,172,385 text bytes,
4,303,829 distinct ngram entries**. Train took 8.688710 seconds, peak RSS
285,888 KiB. Full grammar evaluation took 0.744096 seconds / 245,028 KiB;
full reading 0.592153 seconds / 244,856 KiB. Resource files contain commands
and exit codes. RSS is measured for the child process; every operation was
also contained by the outer 8 GiB/zero-swap worker scope. Builds/training
additionally used the required inner 8 GiB/zero-swap scopes, own process
groups and 600-second kill timeouts. Cargo compilation used one job, tests
four threads. No `/tmp` storage used.

Cache: `/home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability`.
Model `byte6.bin`: **68,861,296 bytes**, SHA-256
`fb187a60997ea56bb9433f1e9ccd9ab81c571d519e648523750277b865999c93`.
Evaluated binary: existing target directory's `release/reading-probability`,
10,932,568 bytes, SHA-256
`3ef857b0d211d3994c03f5d6cbfe112589282cda503533e4f8d46c85dd2b35f9`.
`artifacts.json` records absolute paths. Corpus and model remain in cache,
not git. Artifact sorting makes serialization independent of HashMap seeds;
platform libm differences could cause tiny score rounding differences.

## Validation, reproduction and rejected attempts

Passed: two focused Rust tests (normalization and positive unknown-byte/EOS
probabilities; exact scoring/deterministic serialization round trip and
corrupt/truncated-file rejection); package `cargo fmt --check`; package
all-target Clippy with `-D warnings`; optimized probability binary build.
No giant suite run. The parser source is unchanged and all-target Clippy
checks it; its historical parse metrics were not remeasured.

No rejected model variants or dev-derived tuning: this was the single fixed
experiment. An initial resource-measured train invocation exited 127 because
`/usr/bin/time` is not installed, before training began. Replaced only the
resource wrapper with Python `resource.getrusage`; no model change.

From the worker worktree, reproduce every build/test/train/sample/full step:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/reading-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/reading-run/target
export READING_CACHE=/home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability
bash tools/reading/reproduce.sh
```

The script explicitly lists six training files, uses no API/network calls,
sets scoped resource bounds, tests with `-j1 --test-threads 4`, and writes
new aggregate evidence only to the run cache. It retrains/overwrites its own
`byte6.bin`. `prepare.py` hashes/extracts only authorized training members;
`evaluate` never updates counts. Inspect scores for a supplied string with:

```bash
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  "$CARGO_TARGET_DIR/release/reading-probability" score \
  --model "$READING_CACHE/byte6.bin" --text 'A short example.'
```

No running owned jobs remain. Coordinator can integrate the owned uncommitted
files; no shared progress manifest or scoreboard was modified by this worker.
