# Ask verified apply — frozen plan

2026-10-11. One bounded worker batch; base e0b042a0528a292cc60a08d157e640fa7892588f.
Hypothesis: routing only existing ask rename proposals through World::apply before
rendering, and replacing the recorded reply with the actual result, prevents
unverified writes, rule bypass, mixed JSON output and stale session proposals.
No new automatic remedy/action support. Preserve nested Assuming wrappers.

Acceptance (frozen before runtime edits): authored valid struct/function/field
renames compile and succeed; broken packages which still yield proposals fail
with all source bytes unchanged; disabling rust.apply.changes refuses unchanged;
preview remains read-only and byte-identical to baseline; successful JSON is
exactly one final checked outcome, failed apply never claims success; successful
saved session has Applied rather than pending Proposal, and subsequent yes does
not reapply. Trace contains the actual final reply and verification rules.
Also cover source routing, declared-type behavior, multi-file rollback and stale
proposals with original public fixtures. No test assertions will be weakened.
Fixed sample: struct success, broken package, disabled apply, preview. Then full
authored CLI set on both binaries; scoped unit tests, fmt, clippy and CLI build.
Full 765 benchmark is coordinator-owned and outside this worker's scope.

Baseline: ../askverify-run/baseline/glassmind, SHA256
b53e5f21899189d7e51d8a8ce87aeab561ea92e3196d949cb6ff20d37661e133.
Recorded source e03437a81f2d047babad5e122aec99a88b1c56af; supplied provenance says
runtime unchanged through current base. No external data/dependencies. Fixtures
and adapter are original source, MIT OR Apache-2.0 per repository. Artifact and
source hashes, exact bytes and rejected experiments will be recorded at handoff.

Every cargo invocation uses existing CARGO_TARGET_DIR, disk TMPDIR, systemd user
scope MemoryMax=8G/MemorySwapMax=0, setsid --wait timeout --kill-after=10s 600,
-j1 (except fmt), --test-threads 4 for tests. Existing World wrapper handles
subprocess cargo. Outer scope independently confirmed 8589934592/0. No agents,
model calls, credentials, hidden tests, official test splits or old private logs.
