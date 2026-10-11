# Bounded field-owner batch

Status: local acceptance gate PASSED; uncommitted, coordinator integrates.
Base: `5f2859a730f6d20a0ac64ca9a7551821ed123e86` (world inherited from validated 1331ac7).
Final binary SHA256: `19d2770f8a43cdd9037b8a46747d5a428489b567dcdfd63e40e1bc1e011fb621`
(127,487,680 bytes, debug build). Baseline SHA256:
`ca120ec676a8979d9116ef10380e58cd6f7f3d033fc97ade968a7d510c5b1bd7`.

Run directory: `/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run`.

## Hypothesis and gate

Before runtime edits, the worker stated that a complete owner-aware position audit
could rename one explicitly selected owner's field while retaining homonymous
other-owner fields. Acceptance: useful new exact edits with executed unchanged
before/after behavioral tests, no unsafe newly accepted cases, and all baseline
successes retained. `evaluate.py` and its first-three sample were frozen before
runtime implementation (`FROZEN.sha256`); the full baseline ran before editing.
The original 174 correct / 591 refused / 0 failure full765 baseline is supplied
context, not a result independently reproduced here. Parent full765 remains a
required publication gate.

## Changes

Only owned runtime files changed: `crates/world-rust/src/lib.rs` adds a narrow
homonymous-field route; new `rename_fields.rs` supplies a closed syntax/package
resolver and two unit tests. One unique resolved struct owner, source/package,
and declaration identity are required. Every old-name token must be assigned
by declaration, direct struct literal, or explicit free-function parameter type;
only target-owner positions become edits. No changes to the old rename_at guard,
unique-spelling paths, native checks, isolation, repair, AddField, language,
scorers, manifests, CLI or other tracks.

Unsupported receivers, shadowing, imports/aliases, generics, impls/traits,
closures, shorthand/destructuring old tokens, macros with old tokens, collisions,
raw names, unknown attributes and uncovered package sources refuse. Nested
modules are supported by the existing package graph. This is not general Rust
inference; external crates, cfg test modules and normal assertion macros can
conservatively prevent this new route. Plain prose and strings remain byte exact.

## Measurements

- Frozen sample: baseline 0/3 edits; final 3/3 exact edits.
- Frozen full: baseline 12/16 correct (0 edits, 12 safe expected refusals, 4
  unsupported positives); final 16/16 (4 exact edits, 12 unchanged refusals).
- Every baseline and final fixture executed its original named behavior test
  before and after, with `OWNER_BEHAVIOR_*` stdout markers. Tests independently
  compare both owners' values. Source hashes and full before/expected/after
  files are retained. Exact equality also protects comments, strings and tests.
- Supplemental post-implementation safety audit: 4/4 unchanged refusals on both
  binaries: actual duplicate owner names, import alias, deref impl, inferred local.
  These are safety evidence, not frozen coverage gains.
- Original world baseline: 25 unit + 4 isolation tests passed. Final: 27 unit
  (the same original 25 plus 2 new) + the same 4 isolation tests passed.
- Scoped fmt, clippy with `-D warnings`, and main debug CLI build passed.
- Unchanged regression adapters compare individually, including inherited failures:
  metadata types 35/35; export safety 26/27; ordinary query 8/8; type/edit 9/9;
  variant 14/14; AddField 14/15. Export failure and AddField live-rollback remain
  failures; neither scorer nor fixture expectations were changed to make them green.

The AddField live-rollback failure is inherited source relocation/check isolation,
outside ownership. No full765, workspace-wide suite, release build or CI result
is claimed. Parent must independently replay and preserve at least 174 correct,
zero failure before publishing.

## Rejected/failed experiments

1. Candidate v1 guarded only `PartOf`; v2 added Member-oriented handling
   but still missed the parser's actual `BelongsTo(First)` owner relation. Both
   samples remained 0/3. Diagnostic stderr established the actual typed relation;
   diagnostic code was removed. Candidate v3 reached four exact edits.
2. Final tightening added unique resolved owner validation and import refusal,
   but a missing `RustKind::` qualification failed compilation. A shell sequence
   without `set -e` then copied/replayed the old v3 binary. Files named `final-*`,
   `glassmind-candidate`, `audit-candidate`, and `regressions-final` in the raw run
   are STALE v3 evidence and excluded from acceptance. The alias supplemental case
   exposed this: v3 safely edited it but violated the intended refusal contract.
   Build-error log, failed source, old binary and all replay logs remain retained.
3. Candidate v4 fixed compilation and passed 16/16 + 4/4, but clippy rejected one
   nested if. Final candidate collapses that if without changing its condition;
   it was rebuilt and sample/full/audit/world/regressions replayed. Only
   `glassmind-accepted` and `accepted-*`/`regressions-accepted` are final evidence.

No fixture/expectation or frozen generator edits were used to inflate results.
The exact initial v1/v2 source snapshot was not captured at each build boundary;
retained binaries/logs and intermediate v2 source snapshots document those failed
iterations. Final and failed-tightening source bytes/hashes are preserved.

## Provenance and reproduction

No downloads, external data, additional models, agents, API keys, credentials,
services, held-out items, hidden evaluation code or private logs were used.
New fixture data is locally authored visible development source, version 1,
licensed `MIT OR Apache-2.0` consistently with the repository: 16 frozen input
sources total 6,651 UTF-8 bytes; expected sources total 6,666 bytes. Manifest,
generator, source, binary and evidence hashes accompany this handoff. Supplemental
cases are explicitly post-implementation and have separate hashes. Download size
is zero. No commits were made.

Use the supplied persistent target and disk scratch (never /tmp):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run/target
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run/scratch
bash tools/coding-field-owners/reproduce.sh "$TMPDIR/../replay" "$TMPDIR/../baseline/glassmind"
```

`reproduce.sh` runs scoped fmt/clippy/tests/build, fixed sample then full authored
contracts, supplemental audits, original regression adapters, and an exact outcome
comparison. Every cargo/CLI mutation is within an 8 GiB / zero swap scope with
`setsid --wait timeout --kill-after=10s 600`; cargo uses `-j1` (except fmt), tests
use `--test-threads 4`. Existing adapter CLI subprocesses inherit their outer
scope. `world-compare.sh` reproduces the original world test comparison by
briefly replacing only owned lib.rs from the pinned base and restoring via an
EXIT trap; run it only in an idle isolated checkout. Do not run it concurrently
with compilation/source edits. The consolidated reproduction script was assembled
from executed commands; its entire fresh replay is for coordinator verification.

Raw fixtures, all candidate binaries, native logs and rejected artifacts remain
in the run directory. Reviewable evidence copies and per-case comparisons
are under `reports/`; hash manifest scope is documented there. No worker jobs
are intended to remain running at handoff.
