# ACCEPTED — bounded signed slice-index proof

One bounded batch on base `b27dabe0aeacda2a86acc520450f8df3e59f7c47`.
Uncommitted; no agents, services, model calls, downloads, new dependencies,
credentials, hidden evaluation material, official test splits, or old private logs.
Only `crates/world-rust/src/index_conversion.rs`, minimal registration in `lib.rs`,
and this new owned tools directory changed. Coordinator owns integration and archive.

## Outcome and exact changes

All **3/3 supplied TRAIN requests** now produce the same exact repair:
`players[index]` -> `players[(index) as usize]`. The arithmetic, guards, parameters,
return type, surrounding source bytes, manifests and original lockfile remain intact.
All **7/7 independently authored positives** now receive their frozen exact edits.
All **22/22 negative cases** refuse with unchanged source; zero accepted negatives.
Every positive retains its original E0277 and passes a separate native test covering
negative, zero, valid, last, out-of-range, minimum/maximum signed inputs and empty
slices. These are authored semantic contracts, not merely successful builds.

The new rule `rust.remedy.signed_index` runs only when native suggestions and the
existing Debug helper produced no changes. It proves one immutable signed index's
range and arithmetic through a narrow AST/control-flow model, then obtains an
uncached compiler target-width witness using existing `checks::run_in` with
`cache: None`. The existing tool, copy, isolation, cache, slip/ambiguity and Debug
implementations remain unchanged. The new atomic repair cannot trigger additional
compiler-suggestion rounds and requires all problems resolved.

See [PROOF.md](PROOF.md) for the exact AST language and mathematical argument.
No runtime width check, panic, unwrap, default, alternate return, or error branch
is introduced. Only the index-expression conversion is proposed.

## Frozen plan, baseline and iterations

PLAN and cases were written before runtime edits; their hashes remain unchanged:
- PLAN SHA256 `f3da1fc2bee71ffae85291f7ce715ffdc60268dd7165a35e183806cf0327a236`
- authored cases SHA256 `821cd035801d70f913a23c421cb88fa1f0a7bcabe0c4d72134f47b05d1dfa0f8`

Supplied baseline binary SHA256
`20594291e08e216596645ac21355eca216f8cadfb12cd20e66d7fd71350634f6`.
Baseline first sample: 0/2 repairs, 1/1 safe refusal. Full owned baseline: 0/10
positive repairs, 22/22 unchanged negatives. Coordinator baseline 174/765 passes,
591 refusals, 0 failures is supplied evidence, not a worker rerun or claimed new score.

Iterations (no gate changes):
1. Initial compilation failed because `syn::Error` is not PartialEq; integer parsing
   comparison changed to `.ok() == Some(n)`. No executed runtime from this build.
2. First executed candidate: sample 0/2 repairs, safe refusal 1/1. TOML Value's
   FromStr path rejected the manifests; corrected to `toml::from_str`. Failed binary
   retained at `OWNRUN/failed-first-glassmind`, SHA256
   `e67eb4930137c6a6a0e254a51c8bd82321a61e6cfdb8fdec7ca0858fc34ff4fe`.
3. Corrected sample: 2/2 repairs +1 safe refusal. Added explicit AST budgets,
   shadowed constructor refusal, uncached width proof, and improved scoped tests.
4. Candidate sample 3/3 and full 32/32: 10 exact native-tested repairs, 22 refusals.
   Binary `31f1447da803416a64f86bf7c23525c6cf853002dd1455660fe61b24a9969829`.
5. Final source changed only comments and strengthened a test's diagnostic witness;
   final rebuilt binary replayed sample then full: identical 3/3 and 32/32 outcomes.
   The retained accepted binary and final-full results below are authoritative.

Rejected approaches: blind E0277 casts; host-width inference; general Debug/Hash
changes; permissive calls/aliases/joins; changing guard, rank arithmetic or defaults.
The first TOML implementation was rejected by the frozen sample, not accommodated
by lowering acceptance. Native witness arrays requiring 32 bits pass; the same
source requiring 128 bits fails E0308, an independently executed negative control.

## Validation and retained regressions

- Final world tests: **30 unit +4 isolation integration tests pass**; 0 doc tests.
- `cargo fmt --all -- --check`, scoped world Clippy `-D warnings`, main debug build pass.
- Final sample: **3/3**; final full owned dev: **32/32** (3 supplied +7 authored
  positive repairs; 22 negative refusals).
- Existing diagnostic/Debug adapter **8/8**; metadata types **35/35**; ordinary
  query regression **8/8**; ask/apply contracts **16/16**.
- Export adapter remains **26/27**: inherited `unsupported_type` expectation fails
  because the existing engine now exports the type. Definition unchanged.
- AddField remains **14/15**: the unmodified full adapter crashes on inherited
  `live-rollback` after unexpectedly editing an expected-refusal case. Original
  exception and workspace preserved. Frozen baseline reproduces the same crash.
  Running the other 14 existing cases via its `--cases` option gives 14/14.
  The crashed case is not reported as a pass or dropped from the denominator.

Regressions before the final comment/test-only rebuild used binary `31f144...`;
final 32-case replay and ask/apply used `efe96b...`. Runtime implementation bytes
are unchanged by that rebuild apart from documentation/source positions. All
raw commands/stdout/stderr, original diagnostics and exact before/expected/after
sources are retained under `reports/`; complete original run artifacts remain
at `OWNRUN=/home/gsc/.cache/glassmind/continuous-20261010/indexproof-run`.

## Source, data and identity

Accepted debug binary: `OWNRUN/accepted-glassmind`, SHA256
`efe96b9a0f5cca75bba0a5b5711acbea307426d1b81e484e27aef616689e29ae`.
Final-full result SHA256 `3de9c017b237f766935fd26d6c019911e5292ad0be7f4490fd218878dff02b52`.
Width-probe result SHA256 `df5362d7cd142380855aa6556c69f4a0156925ec38edbd2d4d7e1626bdc46a44`.
32-bit witness source (372 bytes) SHA256
`f9880f9cd9e27a52b6166fcfb8f30b9dde9822b891960bf8bebff6877dabca6d`;
128-bit rejecting control (373 bytes) SHA256
`54f05a513e6fdccd52d1affe503920a0331657e8e7d8840225305ded48953997`.
Runtime patch: `OWNRUN/runtime.patch`, SHA256
`8e94f25e8fce66da5f854637f5d47c3a1df93ded3193331046362dd01ec3f8d2`.
Helper source SHA256 `ea8f06b866bf25443638de82a32e19d33f27b17b55077d26ed51d70e91aff771`;
lib.rs SHA256 `e5f69506f16ca609c7a3d3eba261ca7c9e09ff3169fed129b2dbf1325abba743`.

Public supplied rustbench revision `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`,
MIT OR Apache-2.0, 519 workspace bytes: Cargo.toml 86, Cargo.lock 148, lib.rs 285.
Original provenance independently hash-verified and copied verbatim to
`reports/supplied-provenance.json`. It contains all supplied source hashes and
three original requests; no task.toml/solution/hidden files were accessed.
New authored source fixtures: **29 files /4713 UTF-8 bytes**, MIT OR Apache-2.0.
`provenance.json` records every exact source size/hash and source/binary identity.
Expected sources are frozen separately before any solver invocation. The ten
native behavioral contract files total 3246 bytes; the supplied request JSON is
450 bytes. Behavioral contracts are locally authored and kept separate from solver
output. [COMMANDS.md](COMMANDS.md) records the regression commands and exceptions.

## Reproduction and limits

From the worktree, export OWNRUN above, TMPDIR="$OWNRUN/tmp", and the existing
CARGO_TARGET_DIR="$OWNRUN/target". Run `bash tools/coding-index-proof/reproduce.sh`.
It performs bounded fmt/tests/clippy/build, baseline sample/full, candidate
sample/full and native width-control probes. Output directories must be fresh.
Every Cargo call uses systemd user scope MemoryMax=8G/MemorySwapMax=0,
setsid --wait, timeout --kill-after=10s 600, -j1 except fmt, and test threads 4.
GLASSMIND_SCRATCH is explicitly OWNRUN/scratch. Outer scope verified 8589934592
bytes/zero swap. No jobs outlive handoff.

First sample replays inherited default scratch-target behavior before the adapter
explicitly configured GLASSMIND_CARGO_TARGET to the existing target; all storage
stayed under the owned persistent run directory, never /tmp. Subsequent checks
and final reproduction explicitly use the provided persistent target.

This is a deliberately narrow proof, not general Rust range analysis. Only a
conventional library layout, immutable parameters, straight-line exiting guards
and the modeled arithmetic are supported. Unsupported AST/configuration refuses.
Width is proved for the actual native Cargo configuration checked, including any
inherited target selection; it is not a portability guarantee for a future narrower
target. No cross compiler was installed/tested. Concurrent source/toolchain mutation
is outside the proof assumptions. Behavioral tests are finite, not exhaustive.

Full affected owned dev is complete. Full original **765 DEV** was neither accessed
nor rerun here; the coordinator must independently review/replay all original
cases, retain every prior 174 pass and 0 failures, and handle publishing. No release
build, workspace-wide Clippy, or benchmark-wide score improvement is claimed.
