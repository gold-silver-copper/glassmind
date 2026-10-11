# Syntax-bounded failing-test repair batch

Starting source: `6f50e3e64172bd008b5cbe4611552b69b3b68b60`.
The worker handed changes to the coordinator for integration. No agents, LLM calls,
credentials, hidden tests, expected patches, generators, or private logs were
used. No scorer, CLI, manifest, README, loop, or historical prompt was changed.

## Hypothesis and acceptance (declared before implementation)

Replace line-substring mutation with Rust syntax spans to stop proposing edits
inside strings, identifiers, macros, and explicit test items. Add the bounded
`for` range-boundary operator `..` ↔ `..=`; the permitted moving-average source
exhibited an exclusive upper-bound error. Retain existing verified sample
repairs, verify every returned candidate on a fresh copy, and test exclusion
of non-code text and test bodies. Hidden-test correctness remains unmeasured.

## Exact changes

- `crates/world-rust/src/slips.rs`: syntax visitor using the existing syn
  dependency. Existing binary/boolean operators retain their old per-line
  priority; token spans work without whitespace and with Unicode. Only `for`
  loop ranges gain boundary candidates; slice ranges are unchanged.
- Follow production modules from `src/lib.rs` and `src/main.rs`, skipping
  test-only inline/external modules. Skip explicit `test` attributes and
  `cfg`/`cfg_attr` mentioning test, conservatively including `not(test)`.
  Macros remain opaque. Nonstandard path attributes are conservatively skipped.
- `checks.rs`: replace substring candidate generation; retain the 120-second
  search budget, fresh scratch copies, full cargo-test acceptance, and cleanup.
  Set nested cargo timeout kill grace to the required 10 seconds.
- `lib.rs`: register the private module. Existing application verification and
  rollback remain in force.
- `tools/coding-repair-dev.py`: separate training-only adapter. Labels supply
  kind/split only; stored expected remedies are neither exposed nor used.
  It copies allowed source files, invokes the ordinary CLI, then applies each
  complete nonempty remedy to the untouched copy and independently runs cargo
  test. It reports `verified`, `refused`, and `failed_verification`, never
  equating a visible-test pass with benchmark correctness.

## Development evidence

The initial selection tried the first four `fix_tests` training source groups
plus every seventh `rb_fix_test` source in input order. The non-rustbench harness
source symlinks are dangling in this worker environment, so the frozen available
sample contains four rustbench requests. Full local affected coverage contains
all 27 available unique `rb_fix_test` training sources, first request per source.
Other request styles and build repairs are not included in this local metric.

| Dataset | Baseline verified/refused/failed verification | Candidate |
|---|---:|---:|
| Fixed sample, 4 | 1 / 3 / 0 | 2 / 2 / 0 |
| All available unique training test-repair sources, 27 | 5 / 22 / 0 | 6 / 21 / 0 |

All five baseline verified repairs produce byte-identical edited sources in the
candidate. The gain is the moving-average loop bound. Baseline and candidate use actual
cargo tests; no hidden tests are copied. Baseline binary differs from starting
source only by the required timeout grace. Full candidate may reuse
content-fingerprinted cargo observations from the baseline; final verification
always runs cargo separately after fresh-copy application.

Source version, full-input hash, and exact outcome/change records live under
`/home/gsc/.cache/glassmind/continuous-20261010/coding-run/repair-{baseline,candidate}-{sample,full}/`.
Each directory contains `inputs.jsonl`, `report.json`, per-request outcomes,
and cargo verification logs. Fixed sample input SHA-256:
`e7e40240d859bdc23a6c66f81d4b33d19a420a329743a7a48892b1e6bef5d61b`.
Full input SHA-256:
`6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164`.

Validation: world-rust's seven tests pass (five existing, two new), including
production code after test modules, standalone tests, external test helpers,
strings, identifier substrings, macros, unspaced operators, and Unicode spans.
Targeted clippy with warnings denied, CLI build, formatting, and diff whitespace
checks pass. The full 765-turn rustbench development evaluation belongs to the
coordinator and was not run by this worker. Its prior baseline remains
160 right / 602 refused / 3 wrong; no new benchmark score is claimed here.

## Rejected approaches and limitations

- The first AST traversal used tree order and failed two ordering assertions;
  source ordering fixed those tests. Before final evaluation, retained the old
  operator priority instead: changing priority can select a different
  underconstrained patch that passes visible tests. This is a rejected search
  ordering, not an improvement claim.
- Broad numeric mutation, argument swapping, and slice-range mutation were not
  implemented. This batch adds one small reusable operator, not a search engine.
- Existing visible-test overfitting is not solved: baseline accepted edits in
  leap-year and log-scanning sources contradict their documented behavior.
  These cases demonstrate why the metric is cargo verification, not correctness.
- No custom Cargo target paths, `src/bin` discovery, macro expansion, or
  compiler-resolved conditional module paths. Unsupported syntax/path layouts
  lose proposal coverage conservatively. Feature-based test conventions without
  explicit test attributes cannot be fully identified syntactically.
- The original 120-second search budget is checked between attempts; each
  individual cargo invocation retains its separate 600-second limit.

## Data provenance

No new external data or downloads: **0 new data bytes**. Reused train-v3 requests
and existing training source copies of rustbench revision `f79e71a`, documented
MIT OR Apache-2.0. The 27 selected nonhidden source trees contain 108 files /
29,232 bytes, excluding targets and hidden entries. Full selected input is
9,016 bytes; fixed sample input is 1,321 bytes. Generated train-v3 request data
has no separately declared license in local documentation. These are reused
training-derived development probes, not independent held-out evidence.

## Reproduction

From this worktree, preserving the worker's existing `CARGO_TARGET_DIR` and
non-`/tmp` `TMPDIR`:

```bash
capped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
capped cargo fmt -p glassmind-world-rust --check
capped cargo test -p glassmind-world-rust -j1 -- --test-threads 4
capped cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
capped cargo build -p glassmind -j1
probe=/home/gsc/.cache/glassmind/continuous-20261010/coding-run
capped python3 tools/coding-repair-dev.py \
  --binary "$CARGO_TARGET_DIR/debug/glassmind" \
  --train /home/gsc/.cache/glassmind/data/train-v3 \
  --sources /home/gsc/.cache/glassmind/sources \
  --out "$probe/repair-candidate-sample" --sample
capped python3 tools/coding-repair-dev.py \
  --binary "$CARGO_TARGET_DIR/debug/glassmind" \
  --train /home/gsc/.cache/glassmind/data/train-v3 \
  --sources /home/gsc/.cache/glassmind/sources \
  --out "$probe/repair-candidate-full" \
  --check-cache "$probe/repair-baseline-full/checks"
```

Binary SHA-256 (baseline):
`5849b297b3ce21605328f6da5e0c0e7addbe6da979aa54596886f2bc3c1e1763`.
Candidate:
`691cf90d8dfa3b6827b3ac044534f4b1690ec82d84a097c8598da93b9f5d0279`.

## Coordinator review

The initial visitor guarded test-only items but still traversed test-only
local statements, expressions, match arms and struct field values inside
production functions. The coordinator extended the attribute guards and
the existing adversarial check to cover those cases, including enum
discriminants. All seven focused tests pass. The source above describes
the worker candidate; full rustbench evaluation uses the subsequently
reviewed source and its separately recorded binary hash.

Full rustbench dev at unchanged benchmark commit `f79e71a` completed on
reviewed source `75e3269`: **163 pass, 599 refused, 3 fail / 765**.
All 160 baseline passes remain passes; all three baseline failures remain
failures; 599 refusals are unchanged. The only changes are refusal→pass
for all three styles of `fix-tests/steps-moving-average`. No new wrong
answers or dropped passes. This establishes benchmark improvement beyond
the worker's visible-test measure; no held-out evaluation was used.
`docs/results/coding-syntax-repairs.json` records the exact binary and
result-file hashes and every changed verdict. Workspace checks passed.

Baseline reproduction uses the retained
`$CARGO_TARGET_DIR/debug/glassmind-repair-baseline` with the same adapter and
separate output paths. All scopes/jobs must finish before handoff.
