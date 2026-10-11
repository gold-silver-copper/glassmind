# Unsigned ceiling-division batch

Locally accepted, uncommitted candidate: **public TRAIN 5/27 → 6/27 verified**,
22 → 21 refused, zero failed verifications. All five previously verified patches
are exactly unchanged. Parent integration is conditional on the inaccessible,
opaque full765 gate; this worker did not run or inspect it and makes no claim of
203/765 or retained 200/765. No agents, model/provider calls, services, commits,
downloads, dependency changes, or edits outside the owned directories.

## Frozen plan and exact change

[PLAN.md](PLAN.md) and [fixtures.json](fixtures.json) were generated and SHA256
frozen before runtime edits; `sha256sum -c tools/coding-ceiling-division/frozen.sha256`
passes. All 16 fixture labels and expected edits remained unchanged.

`crates/world-rust/src/ceiling_division.rs` adds syntax-only proposals for one
unsigned primitive quotient, replacing it with the exact fully qualified
`::core::primitive::TYPE::div_ceil(left, right)`. Both operands occur once, in
original order. Concrete parameter types, directly declared same-file struct
fields (including through references), suffixed literals, and unsuffixed literals
anchored by the other known operand are supported. All six unsigned primitives
are covered. Root files only; no inference from task names, English, test answers,
or lookup tables. Comments between quotient operands are conservatively refused.

`checks.rs` appends these proposals after the complete old proposal list, across
all files and lines, including accepted multiplicative swaps. The old sort,
search limits, native verification, scratch copies, and rollback are unchanged.
`slips.rs` only exposes its existing test-attribute predicate within the crate;
`lib.rs` registers the new module. Original protected division-token positions
filter the new visitor. Types/generic arguments, tests and macros are excluded.

`ambiguity.rs` recognizes only the exact fully qualified primitive call, requires
matching unsigned argument/result types, and uses checked quotient/remainder
semantics. Zero divisors, overflow, unsupported calls and shadowing remain
unsupported. Existing unsigned scalar coverage is u8/u16/u32/u64; u128/usize and
struct/multi-parameter functions remain outside that interpreter. Both native
output replays in the original compilation context remain mandatory.

Imports, aliases, custom implementations, generics, local shadowing, nested items,
closures, unknown operands and macro-containing functions are deliberately
conservative refusals. This is unsigned rounding repair, not algorithm synthesis.

## Measured evidence

| Gate | Supplied baseline | Candidate |
|---|---:|---:|
| Exact authored rounding repairs, native verified | 0/8 | 8/8 |
| Negative/failed-verification preservation | 6/6 | 6/6 |
| Legacy native repair | 1/1 | 1/1 |
| Two-quotient refusal with native disagreement evidence | no rounding proposals | 1/1 |
| Fixed public TRAIN sample | 2 verified / 2 refused | 2 verified / 2 refused |
| Full public TRAIN | 5 verified / 22 refused | 6 verified / 21 refused |

The only newly verified source is `rb-fix-tests-shipping-boxes`, with one edit at
`src/lib.rs:13:4` (zero-based character column):

```rust
order.items / order.per_box
// becomes
::core::primitive::u32::div_ceil(order.items, order.per_box)
```

All 108 permitted public source files (29,232 bytes) retain their before-run
SHA256 hashes. The unchanged `tools/coding-repair-dev.py` ran the same ordered
27 sources before and after. Full input SHA256:
`6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164`.
Only public TRAIN kind/split/requests and allowed source/test bytes were used.
No hidden material, expected patches, parent fixtures/results or private history
were accessed. Cached cargo observations were reused by source fingerprint;
accepted patches received native verification through the unchanged runner.

Each positive checks zero numerator, exact/inexact division and native boundaries;
all six width fixtures include maximum numerator with divisor two and an existing
zero-denominator guard. Additional fixtures cover fields and Unicode spans.
The authored harness verifies unchanged original source bytes, one exact expected
expression edit, byte-identical test suffixes and other files, then native tests.

For `f(n:u32) { n / 2 + n / 3 }`, both rounding candidates pass the unchanged
visible assertion `f(7)==6`, but native `f(2)` is respectively 1 and 2. The solver
withdraws the repair. Both variants were also independently replayed against the
unchanged visible test plus their recorded native output assertion; see
[evidence/ambiguity-witness.json](evidence/ambiguity-witness.json) and
[evidence/native-replay.json](evidence/native-replay.json).

The deliberately contradictory rollback fixture rejects the actual primitive
rounding candidate with cargo exit 101, while the original file remains unchanged;
see [evidence/rollback-proof.json](evidence/rollback-proof.json) and its native log.
No rejected runtime hypothesis or fixture relabeling occurred. Initial draft
macro/type/comment restrictions were tightened before the measured candidate.
Signed/custom/unknown cases were excluded rather than widening inference to gain
coverage. The baseline rounding proposal gap is the measured rejected behavior.

Final validation: **43 world-rust tests pass** (37 unit, 1 rounding-order, 4
isolation, 1 multiplicative-order), package/all-target clippy with `-D warnings`
passes, package fmt check passes, real release main builds. The final build and
full test run followed the last source edit. Native compilation on
`rustc 1.99.0 (b940084d7 2026-09-28)` confirms all six primitive methods exist.
No broad workspace or opaque parent evaluation was run.

## Identity, provenance and retained artifacts

Base: `609f42ecdd9071c0888e1f402ee08b8d8616793a` (user's validated runtime source
commit `508b43513868f06cde300dfee1b0a52636d8298d`). The actual supplied baseline main
was used, not a substitute harness.

- Baseline binary SHA256: `7a3d98dfb4eb6428d9d43ceb4cbdb963d20750034d0da223ffaa21dbe9781d81`
- Candidate binary SHA256: `90e463f72105bb3a28fc0b1c17315647af15ba4d4c109dfec0d53b6fd7c67141` (79,736,000 bytes)
- Runtime patch SHA256: `0e9c7a87f68fed355e29c600bc0eb206936b8ceaecebff7d37990b2144bf2ffa`
- Complete crate source manifest SHA256: `c05af5bcc4a3c71be3898cf2b0d58b86a1bc6a2a136609ed5ff2113716ce76f6`
- Frozen authored fixture SHA256: `2d03c1f291e392ca743f55b9ce9aee909013960dc8865c9c5c6924e57ad5b049` (5,113 bytes)
- Frozen PLAN SHA256: `6a3b92ebd030b803ef6621ca002a5b9009d7c0d8e757e36ad6a2c3f4944a73e4`

New data consists only of these 16 locally authored development fixtures and their
generator, created in this batch under the repository's **MIT OR Apache-2.0**
license; the fixture content hash is its immutable version. No external data was
downloaded or newly imported. Existing public `train-v3`/source snapshots were
used read-only; per-file versions and byte counts are in
[evidence/train-source-before.json](evidence/train-source-before.json). No new
licensing claim is made for that existing corpus.

Original candidate source, binary, runtime patch, baseline source archive,
reports, logs, cached check findings, authored source trees and native witnesses
are frozen under:
`/home/gsc/.cache/glassmind/continuous-20261010/ceildivision-run/frozen-candidate`.
The original supplied baseline remains at `ceildivision-run/baseline/glassmind`.
No rollback was needed because all accessible local gates passed. Preserve the
frozen submission before parent revisions or rollback. Per-file source identities
and compact evidence are also retained in this directory's `evidence/`.

## Reproduction and remaining gate

From this worktree, `bash tools/coding-ceiling-division/reproduce.sh` runs the
actual before/after binaries, fixed sample then full27, and package checks into
separate `repro-*` directories. It explicitly exports the supplied persistent
TMPDIR and existing CARGO_TARGET_DIR. Every cargo invocation, including generated
native tests, runs in a systemd 8GiB/zero-swap scope, `setsid --wait`,
`timeout --kill-after=10s 600`, `-j1` (except fmt), and tests use four threads.
The worker's outer scope was independently confirmed as 8GiB/zero swap.

Additional evidence commands (with the same TMPDIR/CARGO_TARGET_DIR exports):

```sh
python tools/coding-ceiling-division/replay-witness.py tools/coding-ceiling-division/evidence/ambiguity-witness.json /home/gsc/.cache/glassmind/continuous-20261010/ceildivision-run/repro-native-replay
python tools/coding-ceiling-division/replay-rollback.py /home/gsc/.cache/glassmind/continuous-20261010/ceildivision-run
sha256sum -c tools/coding-ceiling-division/frozen.sha256
```

Parent next action: review the frozen source and patch, run its independent cases
and opaque full765 runner on the frozen binary, require all 200 previous passes
and zero failures, and check whether this gain covers all three styles. A failed
parent gate requires rejection/rollback while retaining this original evidence.
Bounded scalar disagreement is a veto, not proof of intended semantics or a
complete ambiguity detector. No hidden-benchmark or deployment claim is made.
All owned jobs completed; nothing was left running.
