# Verified enum variant edits: preregistration

Base: 0616eccefac75a538c90a3b9cf3055d828c0058d, 2026-10-11 UTC.
Hypothesis (before implementation): indexing enum variants as owned items and
resolving explicit module/import/impl bindings will enable verified renames of
unit, tuple and record variants without changing unrelated source or behavior.
Acceptance: all fixed positive tiny crates receive the exact declared edits and
retain runtime output; all negative cases remain byte-identical, including failed
verification rollback. Preserve the existing nine type cases and ten world tests.
No benchmark gain is assumed; coordinator owns the full 765-case dev evaluation.

Fixed sample: qualified tuple/record constructors and match arms with comments
and strings; nested external modules with enum and variant aliases; a library
root with inherent impl Self paths and a Unicode variant. Full local cases add
unaliased imports and bare unit patterns, generic enum parameters, ambiguous
variant names, generic owner shadow, type/module collisions, raw identifiers,
new-name collision, macro input, glob imports and failed-build rollback.
The independent literal sources and exact expected edits live in
`tools/coding-variants-dev.py`; they are authored here, not derived from benchmark
questions, expected patches or hidden tests. No downloaded data or model calls.

Rejected approaches before implementation: treating variants as functions,
blanket identifier substitution, accepting a build as proof of binding identity,
and relaxing the existing type resolver's guards.

## Implemented subset and typed semantics

- New `RustKind::Variant` and indexed declaration ownership; `Owner::Variant`
  lookup and existing member/belongs-to relations work with the typed item.
  Variant ownership uses the declaring enum's identity, with refusal for the
  pre-existing file/line identity format's same-line duplicate-enum ambiguity.
- Stable rule `rust.rename.variant`; English noun rules
  `en.rust.noun.variant`, `.variants`, `.enum_variant`, `.enum_variants`.
  No meaning-interface change was needed: its ontology already carries kinds.
- `rename_variants.rs` resolves the selected enum/variant through conventional
  roots (`src/lib.rs`, `src/main.rs`, `lib.rs`, `main.rs`), explicit inline or
  external nested modules, direct explicit imports, enum/module aliases,
  qualified constructors and patterns, and inherent-impl `Self` paths.
  Explicit variant aliases keep their local spelling; import source tokens
  change. Unaliased unit imports and bare unit match patterns change together.
- All old-spelled identifier tokens must have established bindings. Only
  syntax spans change; comments, strings, record fields and unrelated aliases
  remain untouched. Character-based spans preserve Unicode source positions.
- Variant declarations are never classified as functions. Known variant names
  remove speculative function classifications, and caller queries refuse
  collisions with non-function items rather than calling constructors functions.
- Existing `World::apply` builds the edited crate and restores original bytes
  on any failure. No build checker or rollback guard was weakened.
- `rename_types.rs` only exposes its existing module/import helpers within the
  crate. Its accepted syntax and binding guards are unchanged.

## Main CLI integration

The coordinator added `ItemKind::Variant` and the CLI's typed answer mapping.
Three compile-required oracle match arms label the new kind or continue to skip
it in legacy generation. These were mechanical changes from compiler diagnostics;
existing evaluator definitions and generated data are unchanged.

The integrated release CLI passes the fixed3/3 sample, all14 full local cases,
and all9 type-rename cases, sequentially in a dedicated target directory.
Workspace fmt/all-target clippy/release build/strict4/4 fixture smoke pass.
The original worker adapter measurements remain below as historical evidence;
new integration reports identify the actual CLI and source commit `0bff6a1`.
Full765 RustBench dev is complete against the frozen binary: 163 passes,
599 refusals and three failures, with every outcome unchanged. All 81 edit
requests still refuse. No benchmark gain is claimed.
[Full comparison and hashes](results/coding-variant-renames.json).

The worker's `crates/english-rust/examples/coding-edits.rs` adapter remains a
reproduction tool for isolated language/world measurements. It uses the real
parser, lookup, proposal and transactional application, but is not the main CLI.

## Results and rejected experiments

The initial fixed three-crate sample went from 0/3 supported edits to 3/3.
The full independent 14-case set went from 10/14 to 14/14: four new successful
edits, nine intended refusals retained, and one rollback check retained. Every
positive has exact source binding checks, a successful pre/post runtime build,
and identical runtime output. The failed-build case explicitly produces a
proposal, returns `Apply: Err(Unsupported)`, and restores all original bytes.
The old type suite remains 9/9, including its three runtime-positive cases.

The original ten world tests pass, alongside four new variant binding/ownership
tests (14 total). English Rust's 34 original tests plus one typed vocabulary
test pass (35 total). Scoped fmt, all-target clippy with warnings denied, and
adapter build pass. Evidence and source/binary hashes are in
`coding-variants-evidence/`; the final manifest identifies the exact artifacts.

Rejected/invalidated experiments, retained rather than hidden:

1. The first unit-import implementation rejected an unchanged sibling unit
   variant as a local shadow. Fixed by looking that imported name up in the
   explicit enum-variant path set, without relaxing generic or unresolved-name
   guards. The fixed case was already preregistered.
2. Concurrent baseline/candidate runs shared a Cargo target and identical tiny
   package names. One runtime command observed the other case's executable
   (`8` versus `3`); other rows also had contaminating runtime output. That
   measurement is invalid, not a semantic regression or success. Its raw JSON
   is retained as `invalid-concurrent-*.json`. All accepted measurements were
   rerun sequentially. Do not run this harness concurrently in a shared target.
3. Final source review found enum owners with identical file/line/name identity
   could be conflated. Added an explicit ambiguity refusal and an ownership
   regression covering both separate-line and same-line enum declarations.

No train-v3 or other benchmark questions were read. No hidden/held-out items,
expected patches, test generators, evaluator internals or private logs were
read. No scorer, evaluation definition, main CLI, shared manifest, historical
prompt, loop or unrelated track was modified. No agents, model calls, services,
commits or remote actions were launched.

## Limits

This is a deliberately closed subset. Multiple conventional roots, unaccounted
Rust files, cfg/proc/derive attributes, glob or block imports, local items,
unknown macros, old-name macro input, raw old/new names, collisions, same-spelled
variants in different enums, type aliases/reexport chains, qualified associated
type syntax and trait-impl `Self` paths refuse. Only ordinary output macros are
allowed, and any old-name token inside them still prevents an edit. Even an
explicitly selected owner does not bypass same-spelled-variant ambiguity.

Indexed variants can change name lookup/ambiguity outcomes elsewhere. The
coordinator's full765 RustBench dev and transition inspection found no changes.
Whole-workspace build,
clippy and smoke passed after coordinator integration. The generic public
RustItem identity is still file/line based; ambiguous same-line enum owners are
refused rather than inferred.

## Reproduction

Run from this checkout, sequentially. Preserve the existing target directory:

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/variants-run/target
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/variants-run/tmp
capped_cargo() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
capped_cargo fmt -p glassmind-world-rust -p glassmind-english-rust --check
capped_cargo test -p glassmind-world-rust -p glassmind-english-rust -j1 -- --test-threads 4
capped_cargo clippy -p glassmind-world-rust -p glassmind-english-rust --all-targets -j1 -- -D warnings
capped_cargo build -p glassmind-english-rust --example coding-edits -j1
python tools/coding-variants-dev.py --binary "$CARGO_TARGET_DIR/debug/examples/coding-edits" --output "$TMPDIR/repro-variant-sample" --sample
python tools/coding-variants-dev.py --binary "$CARGO_TARGET_DIR/debug/examples/coding-edits" --output "$TMPDIR/repro-variant-full"
python tools/coding-edits-dev.py --binary "$CARGO_TARGET_DIR/debug/examples/coding-edits" --output "$TMPDIR/repro-type-full"
```

The Python tools cap every cargo child using the same scope/timeout, `TMPDIR`
and `-j1`; world verification does likewise through the existing checker.
No test invocation exceeds four test threads. The worker outer scope reports
MemoryMax=8589934592 and MemorySwapMax=0. No storage under `/tmp` is used.
Baseline reproduction uses `$TMPDIR/glassmind-baseline` with the same Python
commands. Its source is the base revision at the top of this report.

## Data provenance

No external data was downloaded: zero new external bytes. The 14 fixed literal
fixture crates were independently authored for this batch and are licensed with
the repository under MIT OR Apache-2.0. Exact original-source UTF-8 byte totals,
fixture script hash, baseline revision, changed-source hashes and binary hashes
are recorded in `coding-variants-evidence/manifest.json`. Expected source lives
only in the dev tool and is never compiled into the runtime adapter or resolver.

## Exact handoff identifiers

- Base source: `0616eccefac75a538c90a3b9cf3055d828c0058d`.
- Uncommitted owned-source digest: `2cef4e14c53ee663ca8ed27a240d0c42fdc9b204c6ee65f5e2c386f0a4225b5c`.
- Candidate adapter SHA256: `10b7e8b9fe12079050674cc1a2de00368d50670e84016437482c94b4dad79726` (50,890,400 bytes).
- Baseline CLI SHA256: `ce16d415cf198bf2e82a1855fb7b482f94bcbd1ee99349636adaddae395c4d94` (108,135,152 bytes).
- New fixture originals: 1,902 UTF-8 Rust source bytes across 14 local cases;
  no external data bytes. The script and each owned source hash are in the manifest.

All cargo/evaluation commands launched by this worker have completed. No owned
agent, service, evaluation or background job is left running. Changes are
uncommitted for coordinator integration; no historical checkpoint was overwritten.
