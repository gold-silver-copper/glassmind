# Accepted after native proof review and full coding DEV

Full coding DEV improves **174 → 177 passes /765**, with 588 refusals and zero
failures. All 174 previous passes remain; the only three transitions are the
chatty, hurried and terse requests for `fix-errors/index-with-i32`. The evaluated
runtime is `0f8dbfa61628e73e7d5cc22bbf81f10feb131a52`, release binary SHA256
`5fcb1577ee7c22487f603aa2dbd7bf39ca60575b218b7c5fde9e507f76c6a815`.
The unchanged runner's full result SHA256 is
`52d3d1a407acb1229b8abaf3fb8d7f2b43591d553a7d9dfc2278d1437dd11e2d`.

The repair changes only the signed index expression to a `usize` conversion.
Compiler diagnostics identify the failing slice index. A bounded AST proof checks
immutable binding identity, dominating exiting guards, nonnegative range and
defined subtraction. A separate uncached native compilation witnesses that the
actual target can represent the signed range. Existing verification must resolve
all build problems; this atomic repair does not trigger extra suggestion rounds.
Guards, rank arithmetic, return behavior and tests are preserved.

Parent review independently froze eleven cases before inspecting the candidate.
Both supported lookups now receive only the expected cast and pass native tests
covering negative/zero/last/outside/extreme ranks and empty slices. Nine cases
with mutation, shadowing, insufficient bounds, non-exiting/nested guards or
unproved arithmetic refuse unchanged. Both the worker binary and integrated
release reproduce these results.

The parent also exercised the installed `wasm32-unknown-unknown` target. Actual
compiler checks accept i16/i32 repairs and refuse i64/i128 unchanged. These four
checks confirm target-sensitive behavior; they do not execute WebAssembly tests
or promise portability to a different future target. The worker's earlier
native 32-bit-required/128-bit-impossible witness controls remain separate.

The final release reproduces all 32 original contracts and exact source hashes:
ten native-tested repairs (three supplied requests and seven authored positives),
22 unchanged negatives. All 34 world tests, workspace formatting, all-target
Clippy, release build and the nonempty four-case smoke gate pass. Inherited
export 26/27 and AddField 14/15 adapter failures remain disclosed; they were not
weakened or counted as passes. Full765 independently validates the final release.

The original 613-file, 1,173,505-byte submission was archived before integration.
All 609 worker manifest entries, runtime/frozen fixture identities and original
binaries were verified. All 612 source/report copies were rechecked before
removing the inactive worktree and temporary login link. Patch differences are
only the old-path `i/` versus `c/` prefix; normalized contents are identical.
Original failed builds and the first nonworking candidate remain preserved.

Raw artifacts: `/home/gsc/.cache/glassmind/continuous-20261010/indexproof-run`,
parent checks under `coordinator/indexproof-review` and `indexproof-independent`,
full benchmark under `coordinator/indexproof-full`. No new dependencies, datasets,
hidden content or scorer changes. Public supplied source and authored fixtures
retain their MIT OR Apache-2.0 notices.

See [proof boundaries](PROOF.md), [full comparison](parent/full-comparison.json),
[independent native cases](parent/independent.json),
[cross-target evidence](parent/cross-target.json) and
[reproduction commands](parent/replay.sh). This is a narrow range proof, not
general Rust dataflow analysis; unsupported source and configurations refuse.
