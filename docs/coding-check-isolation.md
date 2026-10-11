# Native diagnostic check isolation

Candidate on source `9b875b854908cbbd5f5a6c8fce73ddcdbbb8ec8f`, uncommitted for
coordinator review. This is an isolation-only change, independent of rejected
implementation-language work. No benchmark or held-out material was accessed.

## Frozen hypothesis and measured result

`tools/coding-check-isolation/PLAN.md` freezes the authored cases and acceptance
before runtime editing. Baseline source ran `run_tools` at the live root when
`Places.target` was absent. In the exact ordinary-env `repair x` reproduction,
the existing language resolves the request to generic `Repair(Build)`, reports
no applied repair, yet Cargo writes live `Cargo.lock` and `target` entries.

The fixed two-case sample (passing test inquiry and unsupported todo/repair-x,
no lock, default environment) improved from **0/2 to 2/2**. Full authored matrix:
**8/16 to 16/16**. Its axes are passing tests, failing tests, compiler-suggested
missing-mut repair, unsupported todo/repair-x; lock absent/present; optional
owned target/cache absent/present. All package names and root basenames collide.
Each case starts with existing hidden and target sentinels. All live entries and
file hashes are compared, including lockfiles and preexisting artifacts.

Every diagnostic/refusal now leaves all live bytes and entries unchanged.
Every applied compiler repair changes exactly the intended source bytes in
`src/lib.rs`; the compiler error, proposed patch and success response are
unchanged. All 16 baseline/candidate response traces match after replacing only
OS-assigned numeric thread IDs in panic messages. No incorrect repair is claimed.
These are authored development checks, not a new benchmark score.

## Exact implementation

Only `crates/world-rust/src/checks.rs` changes runtime behavior:

- `run_in_until` always copies legitimate inputs using existing `copy_tree` rules
  to a unique scratch path, then runs native tools there. Cleanup removes only
  that owned copy, on both success and ordinary error.
- Unset `Places.target` selects a disposable scratch target explicitly, so
  inherited `CARGO_TARGET_DIR` cannot redirect these checks into the live tree.
  Explicit owned targets still retain builds. Explicit result caches still work.
- Preserve future-dated source copies and content fingerprints. Cache generation
  moves from v5 to v6 so pre-isolation findings are not reused.
- Repair proposals, actual apply, rollback, rules, ambiguity search and the
  Cargo resource/timeout wrapper remain unchanged. Fresh copies also avoid the
  old per-source/per-process copy-path collision between concurrent checks.

`crates/world-rust/tests/check_isolation.rs` adds four public authored native
integration groups: 16-case Test matrix, exact owned-cache hits and uncached
findings; shared target/cache across same-name roots and changed bytes with old
mtime; lint findings; actual approved/stale/rule-disabled/test-breaking repair
and rollback; build-script relative writes, file/directory symlinks, and external
relative dependency refusal. The latter checks that unsupported topology never
falls back to live Cargo execution. Existing 25 world tests include native
ambiguity witnesses, bounded deadlines and source-preserving repair contracts.

## Validation and evidence

All 25 existing world tests plus four new integration groups passed; all three
existing CLI edit contract tests passed. World-Rust all-target clippy with
`-D warnings`, workspace fmt check, and a debug CLI build passed. Final integration
tests and final binary sample/full replay were repeated after formatting/test
module adjustments, with exact binary hash recorded in `evidence/provenance.json`.
No release build, broad workspace tests or full benchmark was run by this worker.

The coordinator must independently review and run full765 before integration:
all 174 prior passes retained, zero failures, all 765 outcomes unchanged. This
worker makes no claim that that gate has passed or that rejected implementation
language is accepted.

Raw results, comparisons and logs live in `tools/coding-check-isolation/evidence/`.
Raw generated trees and binaries remain in
`/home/gsc/.cache/glassmind/continuous-20261010/checkisolation-run/tmp/`.
Baseline `baseline-final`, candidate `final-sample` and `final-full`, final binary
`final-glassmind`. Initial adapter/assertion failures remain separately in the
same run directory and relevant native/clippy failure logs are retained in git.

## Failures and rejected alternatives

One runtime candidate was implemented; no alternate runtime patch was rejected.
The baseline behavior is rejected because it mutates the live tree. Refusing
repair commands, deleting live artifacts, dropping diagnostics and lowering
mtime freshness protections were not attempted.

Two initial adapter assumptions were wrong: repair-x diagnoses Build, not Test;
`do --explain` prints the Remedy proposal then the spoken applied result, not an
Applied debug record. Corrections require the actual baseline contract, exact
source bytes and the same frozen mutation predicate; original runs are retained.
The first native run compared fresh panic thread IDs literally; its single
failure was fixed by masking only numeric IDs for uncached results (cached
results still compare exactly). First clippy reported 18 unwraps in integration
helpers outside a syntactic test module; wrapping tests in `cfg(test)` follows
existing test conventions, without lint suppression or production changes.

## Topology and safety limits

This isolates normal Cargo lock/build output; it is not an OS security sandbox.
Build scripts, proc macros and tests execute with the caller's authority. Relative
writes in a build script go to the copy, as tested; arbitrary absolute paths,
network writes and environment-directed external writes are not constrained.

Copy rules remain exactly the established rules: omit all hidden entries and
`target`; copy other project files and date them one day ahead. Hidden `.cargo`
configuration, ancestor workspace configuration and relative path dependencies
outside the copied tree are not reconstructed. A legitimate external relative
dependency may therefore become an unreadable build failure/refusal; there is no
live-root fallback. Completely contained project inputs can be copied normally.
Absolute paths, `CARGO_MANIFEST_DIR`-dependent behavior, symlinks and external
configuration can make relocation observable; no broad topology equivalence is
claimed. File symlinks are materialized using the existing copy behavior;
directory symlinks fail conservatively. No new symlink traversal is added.

Caller-provided scratch, target and result-cache locations must be owned writable
storage outside the live project. Explicitly pointing an owned build/cache path
into a live tree authorizes writes there and is outside the no-live-artifact
contract. Existing fingerprint/cache limitations around external configuration
and external dependency contents remain. Default copies/targets are discarded;
owned target caches remain, and default checking can cost more compilation time.
Abrupt process termination may leave owned scratch artifacts; ordinary errors
clean them. No live artifacts are deleted.

## Reproduce

No new dependencies or external data. All new fixtures are authored in the new
Rust tests and Python adapter under the repository's **MIT OR Apache-2.0** license.
`evidence/provenance.json` records exact source/generator/fixture bytes and SHA256,
baseline source version and both binary hashes.

From the worktree, preserve the supplied on-disk `TMPDIR` and existing
`CARGO_TARGET_DIR`, then run:

```sh
bash tools/coding-check-isolation/reproduce.sh \
  /home/gsc/.cache/glassmind/continuous-20261010/checkisolation-run/baseline/glassmind \
  "$TMPDIR/replay-new"
```

The script has been syntax-checked; its individual steps were run in this batch.
It runs sample before full, baseline before candidate, no full765. Every Cargo
invocation uses TMPDIR, an 8GiB/zero-swap systemd user scope, `setsid --wait`,
`timeout --kill-after=10s 600`, `-j1` except fmt, and test threads4. Runtime nested
Cargo uses the unchanged wrapper with the same limits and optional shorter
ambiguity deadline. The worker's outer scope was verified at 8589934592 memory
bytes and zero swap. No agents, LLM calls, credentials, services, commits or
background jobs were launched.
