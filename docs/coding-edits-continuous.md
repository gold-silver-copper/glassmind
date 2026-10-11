# Bounded type rename candidate — 2026-10-10

Starting source: `e73f68c36e183b4060468cc6e752b8709f0662a0`.
Integrated coding source: `ea52c58bc1c575db236c71ee435594d65831e96e`. No shared manifests,
CLI, slips repair code, scorers, historical prompts or datasets were changed.

## Hypothesis and fixed acceptance criterion

Stated before implementation: an explicit module/import resolver over syn can
safely add struct/enum renames across declarations, type references,
constructors, impls and imports. Accept only a gain on the fixed positive
sample with identical program output and exact intended source edits, while
all ambiguous/refused or failed applications leave original bytes intact.

The fixed positive sample is `struct_cross_file`, `enum_paths`, and
`unicode_alias`. Full local dev adds `shadow`, `collision`, `same_name`,
`macro`, `glob`, and `rollback`. These are independent hand-authored tiny
crates, not RustBench questions or hidden/generated benchmark material.

## Implementation

- `crates/world-rust/src/rename_types.rs` walks one conventional crate's
  explicit module tree and resolves lexical module paths and explicit imports.
  It accounts for every old-name identifier token, requiring each accepted
  position to bind to the selected declaration. The fresh name must be an
  identifier absent from source tokens, including raw spellings.
- `crates/world-rust/src/lib.rs` routes existing typed `Action::Rename` for
  `Struct` and `Enum` through stable rule `rust.rename.type`. Existing kinds
  and reusable meanings suffice; no English grammar or meaning changes.
- Proposals use existing character-based `RustSpot` coordinates and existing
  verified `World::apply`. The exercised CLI is `glassmind do`, which compiles
  after applying and restores originals on failure. No runtime lookup tables,
  LLM, global text replacement, services or new dependencies.
- Three focused tests cover nested modules, ordinary generics, Unicode
  columns, raw identifiers, associated types, module-name shadowing, macro
  expansion, derives, shorthand fields and fresh-name collisions.

## Measurement

| Fixed development set | Baseline | Candidate |
|---|---:|---:|
| Positive sample: applied, exact source, same output | 0/3 | 3/3 |
| Full local dev: all case acceptance checks | 6/9 | 9/9 |
| Unsupported/failed cases: unchanged original source | 6/6 | 6/6 |
| Successfully renamed token positions | 0 | 15 |

All three accepted crates compile and execute before and after; stdout is
identical. Comments, strings and the Unicode example's explicit import alias
remain unchanged. Both enum constructors and match patterns are exercised.
The rollback case starts with an unresolved function: the candidate proposes
two edits, compilation fails, `do` reports failure, and source bytes match
the original. This case is not counted as a working program.

Final fixed sample was run before final full local dev. Raw per-case replies,
exit results, source checks and binary hashes are in
`docs/coding-edits-dev-results.json`. Checks passed:

- world-rust library tests: **10/10**, including all seven existing tests;
- world-rust all-target clippy with `-D warnings`;
- world-rust fmt check;
- CLI debug build and the nine independent CLI/compile/run examples.

Coordinator reproduction passes all nine local cases and required workspace
checks. The full765 RustBench dev comparison preserves **every outcome**:
163passed,599refused,3failed, including all81 benchmark edit requests still
refused. This is a useful bounded capability on the separate local examples,
**not a RustBench gain**. [Full comparison](results/coding-type-renames.json). No train/held-out items, task generators, official test
split, private logs or credentials were read. No new model calls were made.

## Rejected approaches and review findings

`rust-analyzer` resolves to `/home/gsc/.cargo/bin/rust-analyzer`, but
`rust-analyzer --version` reports that stable has no such binary. An analyzer
installation and LSP integration were not attempted within this batch; no
analyzer process was launched. Chose the smaller explicit semantic subset.

Rejected syntax-only path classification because a generic can shadow a
module and `<T as Trait>::Old` can name an unrelated associated type. Those
forms refuse. Review also excluded raw spelling inconsistencies and shorthand
fields, whose single token can name both a constructor and a field. Focused
regressions pass. No measured candidate regressed the fixed sample. One initial
build command used nonexistent package `glassmind-cli`; corrected to
`glassmind`, with the same required resource wrapper.

## Limits

The subset intentionally refuses multiple lib/bin roots, extra unaccounted
Rust files (including integration tests/build scripts), unreadable/unparsed
sources, nonstandard module paths, glob imports, duplicate names, nested block
items/imports, raw old/new spellings, qualified associated types, conflicting
generic bindings, most attributes (including cfg and derive), and macros other
than unshadowed standard println/eprintln. Even output macros refuse if they
contain the old identifier. Import chains/reexports are only accepted when
the explicit lexical path resolves directly. Variants themselves are not
rename targets. These restrictions trade coverage for explicit bindings.

External/downstream crates and name-reflection behavior are outside the closed
source guarantee. Application verifies compilation, not arbitrary program
semantics; the independent runtime/source checks provide additional evidence
for these examples only. At the time of this experiment, `ask --apply` used
the CLI's older direct edit route. A later independently verified change now
routes it through `World::apply`, with build checks and rollback:
[current application contract](../tools/coding-askverify/PARENT.md).

## Provenance and reproduction

New development data: original examples authored for this batch, version
`coding-edits-dev-v1`, repository license **MIT OR Apache-2.0**. Nine crates,
12 Rust files, **1,191 UTF-8 source bytes**. The fixed script is **4,698 bytes**;
its SHA-256 and evaluated binary hashes are recorded in the JSON evidence.
No downloaded data or dependency changes. The examples and their explicit
acceptance checks are development-only, never runtime inputs.

Run from this worktree with its existing target directory. All cargo
invocations, including the script's per-example cargo runs and world checks,
use memory scopes, setsid, timeout, disk TMPDIR and one compilation job.

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/edits-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/edits-run/target
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build -j1 -p glassmind
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test -j1 -p glassmind-world-rust --lib -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy -j1 -p glassmind-world-rust --all-targets -- -D warnings
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt --package glassmind-world-rust --check
python3 tools/coding-edits-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/rename-repro-sample" --sample
python3 tools/coding-edits-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/rename-repro-full"
```

Baseline binary retained at `$TMPDIR/glassmind-rename-baseline`; invoke the
same script against it with a separate output directory. Final measured
artifacts are `$TMPDIR/rename-final-{sample,full}/results.json`. The outer
`glassmind-continuous-edits.scope` was verified MemoryMax=8589934592 and
MemorySwapMax=0. All owned commands finished; no services or LSP processes
were started or left running.
