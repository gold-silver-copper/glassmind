# Ask verified apply handoff

Completed one bounded reliability batch. Uncommitted; coordinator integrates.
Base: `e0b042a0528a292cc60a08d157e640fa7892588f`.

## Exact product change

Only `crates/cli/src/main.rs` and `crates/cli/src/edit.rs` changed at runtime.
`ask --apply` now sends the original typed rename proposal, including nested
Assuming replies, to the existing `World::apply` before rendering. It honors
`rust.apply.changes`, stale edits, cargo verification and rollback. No new cargo
runner or verification implementation was introduced in product code. Removed
unused unsafe direct writer; its stale/all-or-none coverage was replaced with a
multi-file stale check through the real world, plus verified success coverage.

Successful output uses existing Applied English / Checked JSON. JSON stdout is
one final outcome; failure is a Refuse outcome plus diagnostic stderr and exit 1.
Session's current turn and final trace reply are updated to Applied or Trouble;
Applied stops pending-history lookup, including when an earlier preview remains
in history. Verification rules are included in trace/outcome. Existing proposal
trace entries describe the proposal stage; the final trace reply is the result.
Only old Response::Edit eligibility is applied: neither complete nor incomplete
Remedy replies acquire automatic apply behavior. Ordinary preview and declared-
type paths remain unchanged. No other CLI features/world code/manifests changed.

## Measured evidence

| Authored public contracts | Baseline | Candidate |
| --- | ---: | ---: |
| Fixed sample (subset of full) | 1/4 | 4/4 |
| Complete primary set | 6/16 | 16/16 |
| Supplemental set | 1/4 | 4/4 |

20 distinct CLI contracts total: **7/20 -> 20/20**. This is scoped product
reliability evidence, not a benchmark-score or generalization claim. Full 765
coding dev remains coordinator-owned and was not accessed/run here.

Cases include valid struct/function/field renames, broken-package rollback,
multi-file rollback, disabled rule, JSON and plain previews, JSON with explain,
actual final trace, successful and failed saved sessions, subsequent yes, existing
preview history, two named source routing, ordinary query, declared-type success
and refusal/session guard, and retained macro/cfg rename refusals.
Six primary non-application stdout comparisons are byte-identical baseline vs
candidate. Every original fixture source/manifest hash rechecked unchanged.
Raw exit statuses, stdout/stderr, parsed JSON, session bytes as parsed structures,
before/after source hashes and commands are in `reports/*{full,sample,supplement}*.json`.

Validation: `cargo test -p glassmind -j1 -- --test-threads 4` 10/10 (three new
focused tests replace one obsolete direct-writer test); clippy all CLI targets
with `-D warnings`; CLI fmt check; final CLI debug build; original field fixture
assertion after rename 1/1. All exit 0. Logs are in `reports/`. Final rebuild binary
hash equals the already evaluated candidate, so no redundant rerun was needed.

Frozen baseline: `../askverify-run/baseline/glassmind`, 74,356,104 bytes,
SHA256 `b53e5f21899189d7e51d8a8ce87aeab561ea92e3196d949cb6ff20d37661e133`.
Supplied provenance source `e03437a81f2d047babad5e122aec99a88b1c56af`, explicitly
runtime unchanged through current base. Hash independently checked.
Candidate: `../askverify-run/candidate/glassmind`, debug build,
SHA256 `b94d7bba5d1fa60194d2ffa61d1af12523426458c5df4282b4be01cf4b25ec84`.

## Sources and reproduction

No downloaded data or new dependencies. Public fixtures are original authored
source v1: 11 files, **1,570 bytes**, licensed **MIT OR Apache-2.0** per repo.
`provenance.json` records every file's bytes/hash. Adapter and new unit-test
source are original under the same license. `EXPERIMENTS.md` records rejected
approaches, retained preflight refusals and the pre-candidate correction of an
incorrect English wording assertion. Original adapter and raw baseline results
are preserved; the corrected baseline still scores 6/16. Assertions in the
original Rust fixture were never changed or removed.

From repo root, using the supplied target and disk TMPDIR:

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/askverify-run/target
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/askverify-run/tmp
bash tools/coding-askverify/reproduce.sh \
  /home/gsc/.cache/glassmind/continuous-20261010/askverify-run/baseline/glassmind \
  /home/gsc/.cache/glassmind/continuous-20261010/askverify-run/replay-new
```

Output directory must be new. The script reproduces the individually completed
commands; the consolidated script itself was syntax-checked, not rerun end-to-end.
Each cargo call uses systemd-run --user --scope -pMemoryMax=8G
-pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600, disk TMPDIR, existing
CARGO_TARGET_DIR, -j1 (except fmt), and --test-threads 4 for tests. Product calls
use the existing world wrapper with these same constraints. Outer scope was
independently confirmed at 8,589,934,592 bytes and zero swap.

## Limits and handback

Verification preserves the world's existing cargo-build semantics for renames;
it does not claim behavioral equivalence or strengthen world verification. No
rollback-failure filesystem fault injection, crash atomicity or session-save I/O
failure handling was added. An unsuccessful apply records Trouble, not success;
an older pending proposal can remain retryable after failure under existing
session semantics. All successful writable-session cases clear pending state.

No official test splits, hidden tests, expected patches, old private logs,
runtime LLMs, answer tables, keys or credentials were accessed. No additional
agents, model calls, services or persistent jobs were launched. The worker's
inherited model/login configuration was not changed or independently inspected.
No commits. Runtime patch and full owned submission are preserved in
`../askverify-run/submitted/` with SHA256 manifest; no restores were performed.
All launched command sessions finished. Coordinator should review this exact
contract and run full affected coding/regression before integration/shipping.
