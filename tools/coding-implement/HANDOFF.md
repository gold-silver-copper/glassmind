# Handoff: REJECTED runtime, evidence only

One bounded experiment completed at source **35d3ea80d63791af1625d5019077d6ea2cfa5f26**.
No commit, agent, LLM call, service, hidden evaluation access, dependency change,
manifest change or benchmark scorer change. All runtime source is restored to
HEAD. Only this new tool/report directory and `docs/coding-implement.md` remain.
Do **not** integrate `rejected.patch` as an accepted capability.

## Outcome and acceptance

Prospective hypothesis/budget/gates: `PLAN.md`. Frozen authored fixtures:
`fixtures.py`, 3,433 bytes, SHA256
`7284edba6c1fcc4b55f432069c3c74a30f69d781c558985b73c9b4f0b267aa40`.
Fixed sample: baseline **0/3** implementations, candidate **3/3**. Full authored
suite: **20/20 checks**, comprising **6 independently tested implementations**
and **14 safe refusals**. The same signatures support different behaviors;
identifiers include Spanish and Japanese. Eight candidates passed the deliberately
insufficient test, causing refusal. Conflicting tests, no tests, uncalled target,
shadowed type/trait/macro, existing/partial bodies, unsupported signatures,
malformed requests, missing target and compile failure all refused unchanged.
The live-rollback fixture passed synthesis checks, then failed its original test
in the live root; original source/manifest bytes were restored and Cargo.lock removed.
An additional missing-Cargo probe refused unchanged. Three new Rust tests cover
syntax consumption, signature-only generation, stale application and disabled rules.
All **76 scoped tests** passed (10 CLI, 39 English-Rust, 27 world-Rust).
Affected-package all-target clippy, workspace fmt and debug build passed.

**Release gate failed by demonstrated request-coverage limitation.** After the
local suite, the permitted TRAIN input syntax audit found **0/60 accepted request
forms**. Terse requests use backtick-quoted names; this parser accepts only bare
names. Other forms require more language coverage. This is a syntax audit, **not
an execution or scoring of the 60 tasks**. The family also cannot express most of
the requested algorithms. No full765 run was performed; there is **no claimed
177-pass result or improvement to the reported 174/591/0 baseline**. The runtime
was rejected without asking the parent to spend a full benchmark run on it.
A new API attempt needs a new prospective experiment; the acceptance gate was
not reduced to the local six positives.

## Exact archived changes

`rejected-source/` contains exact copies of all five changed runtime files:

- `crates/world-rust/src/implement.rs`: signature-directed typed pipeline generator,
  conservative closed-package checks, native nonempty failing baseline, complete
  finite-family trials, multiple-survivor refusal, evidence logs, stale/package/rule
  validation, fresh native application check and rollback.
- `crates/english-rust/src/implement.rs`: fully consumed `implement NAME` parser;
  only action and target pass to the world.
- `crates/world-rust/src/lib.rs` and `crates/english-rust/src/lib.rs`: exports only.
- `crates/cli/src/main.rs`: minimal `do` route beside AddField, honest test-verified
  wording. No ask/session/repair/mutation/rename/check behavior was edited.

The complete reversible patch is `rejected.patch`, 28,680 bytes, SHA256
`32618f996cfa2d2170aa1f955987e641ce9e5f81b198560968233c4326709d84`.
World implementation SHA256:
`e41e7cf852151eb02a692e362914593e3a33c3bea9567706997f52711c4e9697`.
Bridge SHA256:
`69bee0e9e6e8857356fc6f9a09aa4ec025579a7b8a55dc8adb13ce742cb7cf99`.
All five source hashes/lengths are in `reports/candidate-source.json`.
Source copies and patch hashes were checked **before restoration**. The rejection
report was written before restoration; its hash was finalized afterward in
`MANIFEST.sha256`. `git apply --check rejected.patch` passes on restored HEAD.

Candidate binary retained at
`/home/gsc/.cache/glassmind/continuous-20261010/implement-run/glassmind-candidate`,
SHA256 `d80716b6411c99b8ebe6491e90906f660cc87fd576e2eb6a3bfa144b1856e952`.
Restored baseline binary is the neighboring `glassmind-baseline`, SHA256
`a4b934d872f170eea32efab318223d8ebb9174acccc6158201d25407594253e4`.

## Evidence and reproduction

`reports/validation.json` summarizes results. `reports/sample-final/results.json`
and `reports/full/results.json` contain all per-case hashes/timings/verdicts;
corresponding scratch evidence retains each generated candidate, native stdout,
stderr, status and runtime. Independent verification logs are alongside case logs.
`reports/tests.log` records all76 tests. `reports/train-syntax.json` records the
request-syntax audit; `coverage.py` reproduces it from TRAIN kind/split/identity and
input text only, without accessing expected values. The authored evaluator never
passes fixture checks or desired outputs to the runtime beyond original project tests.

From the repository root, with fresh output directories:

```bash
RUN=/home/gsc/.cache/glassmind/continuous-20261010/implement-run
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
export GLASSMIND_SCRATCH="$RUN/replay-scratch"
sha256sum -c tools/coding-implement/FROZEN.sha256
python3 tools/coding-implement/evaluate.py --binary "$RUN/glassmind-candidate" --out "$RUN/replay-sample" --sample
python3 tools/coding-implement/evaluate.py --binary "$RUN/glassmind-candidate" --out "$RUN/replay-full"
python3 tools/coding-implement/evaluate.py --binary "$RUN/glassmind-baseline" --out "$RUN/replay-baseline" --sample
# Baseline exits 1: all three positive expectations correctly remain unmet.
python3 tools/coding-implement/coverage.py
```

To rebuild the rejected candidate **only in a disposable checkout of the source
revision**, apply the archived patch, retain the existing target directory and use:

```bash
git apply tools/coding-implement/rejected.patch
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt --all -- --check
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind-world-rust -p glassmind-english-rust -p glassmind -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy -p glassmind-world-rust -p glassmind-english-rust -p glassmind --all-targets -j1 -- -D warnings
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build -p glassmind -j1
```

All actual cargo invocations used the existing target, TMPDIR, 8GiB/zero-swap
scopes, setsid --wait, timeouts, -j1 except fmt, and four threads for tests.
No owned cargo processes or independently surviving jobs remain.

## Limits, rejected attempts and deviations

- The first sample's nested runner used `--kill-after=1s`; corrected to the required
  10s before the final sample/full run. Initial sample logs remain in `reports/sample`.
- The first filtered Rust test used the inherited default scratch base outside the
  owned run root. Its owned evidence directory `implement-3392036-1.evidence` was
  moved into the run root; subsequent full tests explicitly set GLASSMIND_SCRATCH.
  No unrelated scratch contents were inspected or changed.
- Initial clippy rejected five indexing expressions; replaced with checked pattern
  matching/access, then all-target clippy passed. The first baseline archive build
  omitted tools workspace members and failed before compilation. Its log is retained;
  baseline was then built from the restored full source and measured successfully.
- The baseline's existing generic command path created Cargo.lock in all three
  sample projects. Source bytes stayed unchanged. The evaluator reports these
  baseline checks as unmet, not successes. Existing behavior was not changed.
- `reports/FROZEN.original.sha256` includes an early pre-format generator-source
  hash; that draft is not the final source. `FROZEN.sha256` verifies the identical
  frozen fixture definition; final candidate hashes are separately recorded.
- Conservative multi-survivor refusal does not prove uniqueness over Rust programs.
  Unsupported package/type/equivalence cases refuse; no pure-interpreter witnesses.
  Native tests can have arbitrary external side effects, and restoration is not
  crash-atomic. Adversarial filesystem changes, hostile fake test output, disk-full
  restoration failure, and process-tree timeouts were not fault-injected.
- No JSON extension, release build, full765 regression, external source-signature
  study or hidden evaluation was performed. The explicit route returned ordinary
  AddField-style text, including refusals with exit0; that contract was not recast.
- No downloaded dataset/model. New fixture data is locally authored on2026-10-11,
  source version identified by the frozen hash above, under the workspace's
  **MIT OR Apache-2.0** code license. TRAIN request audit uses the existing local
  train-v3 inputs only; no external license or new source revision is invented.
