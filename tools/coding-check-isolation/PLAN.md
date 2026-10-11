# Frozen before runtime editing: 2026-10-11

Base: 9b875b854908cbbd5f5a6c8fce73ddcdbbb8ec8f. Hypothesis: diagnostic
checks always using a fresh scratch copy eliminate live Cargo.lock/target writes
with GLASSMIND_CARGO_TARGET unset, without changing native findings or repair.

Authored dependency-free development matrix: pass, failing assertion, missing-mut
compiler suggestion, unsupported todo; Cargo.lock absent/present; default versus
caller-owned target/cache. Preserve existing target and hidden sentinels. All
packages and source basenames intentionally collide. Fixed sample: default,
lock-absent pass and todo. Full: all 16 combinations. Inspect all live bytes and
entries, allow only the explicit compiler repair's src/lib.rs change. Native
checks add cache freshness, applied/stale/failed/rule-disabled proposals, and
symlink/copy behavior. Existing ambiguity tests remain unchanged.

Acceptance: every authored case passes; repair-x mutation disappears; known
pass/fail diagnostics remain correct; no false applied repair; relevant Rust
tests, fmt, clippy, build pass. No evaluation definition changes. Parent alone
runs full765 (retain174 old passes, zero fail, all outcomes unchanged). This
separate isolation patch cannot accept rejected implementation-language work.

No network/data/LLM calls, services, background workers, agents or commits.
New fixtures are authored here under repository MIT OR Apache-2.0; record exact
bytes/hashes and commands at handoff. Every cargo uses TMPDIR, existing target,
8GiB/zero-swap scope, setsid --wait, timeout --kill-after=10s 600, -j1, test threads4.

Baseline sample protocol correction BEFORE runtime edits: repair x resolves via
existing generic Repair(Build), not Repair(Test). Its expected unchanged outcome
is Remedy(Build, no changes/fixed/left), not a named failing test. Native Test
coverage separately requires todo's failing test. No mutation assertion weakened.
Second adapter assertion correction: baseline do --explain prints the proposal's
Remedy trace followed by 'I made one change. No problems with the build.', not an
Applied debug trace. Require that exact success line, E0384, and exact intended
source replacement. Raw baseline-full with the original assertion is retained;
replay the frozen baseline with the corrected reporting predicate before scoring.
