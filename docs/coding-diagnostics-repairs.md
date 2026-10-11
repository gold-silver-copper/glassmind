# Compiler-reported standard Debug derivation

Preregistered before runtime edits, 2026-10-10. Base source
`05e80930ef2b06e8e054ece3ab4cc69666d19b98`. Supplied baseline CLI from
`0bff6a1`; coordinator states repair code identical to base. Scope: repair
coverage only; no rename changes, scorer changes, dataset downloads or LLM calls.

Audit fresh copies of all six permitted workspaces with capped cargo check JSON.
Rejected hypothesis H1: recursively reading diagnostic children alone would
repair these cases. No eligible nested replacements occur. Collection annotation
has `HasPlaceholders`; missing match arm inserts `todo!()`. Neither is enabled.

Selected H2: rustc E0277 explicitly naming a local type missing Debug can drive
one bounded AST operator: add qualified standard Debug derive to the unique
eligible struct or enum. Preserve existing fields, functions, derives, and tests.
No other trait additions or speculative alternatives. Only fall back when no
existing compiler edits are available; verify the complete requested check in
scratch and refuse on any remaining error. Stable rule: rust.remedy.derive_debug.

Fixed sample: terse prompts of derive-debug, collect-needs-type,
match-missing-variant, in that order. Full affected development set: all six
supplied workspaces, each of its three supplied prompts (18). Baseline measured
before runtime edits: 0 changed/18; all remain refused or clarification. Sample
0/3. Baseline stdout and fresh compiler JSON retained under worker TMPDIR.

Acceptance: at least one substantive previously refused provided-dev repair,
all existing syntax repair tests retained, exact independent functional checks,
and no old correct/refused becoming wrong in coordinator's full765 parent gate.
Worker cannot claim final gate until coordinator runs it.

Predeclared independent functional check for the affected supplied workspace:
original binary output exactly `[Reading { sensor: "attic", celsius: 18.5 }, Reading { sensor: "cellar", celsius: 12.0 }]\n`.
A separately added verification example must output `None|Some("hot")|Some("first")\n`
for empty readings, unequal temperatures, and a single reading respectively.
Checks are added only AFTER the repair request, never exposed to proposer.
Own fixtures: independently named record, tuple, enum, Unicode placement;
refusals for ambiguous names, cfg/test items, non-Debug fields and an unrelated
compile error. Failed verification must leave original source bytes unchanged.

## Final handoff

Uncommitted bounded candidate, no integration/commit performed. Owned changes:

- `crates/world-rust/src/standard_traits.rs`: recognizes only compiler E0277
  messages of the exact form `` `Name` doesn't implement `Debug` ``; parses the
  name as an identifier and scans declared production files for one local
  struct/enum. Duplicate declarations and aliases, generic/qualified diagnostic
  names, conditional/test/unknown item attributes and external symlink targets
  refuse. Existing derive attributes must have standard built-in spellings.
  Generates only `#[derive(::core::fmt::Debug)]`; the complete original item is
  the replacement anchor, preserving Unicode character coordinates and rejecting
  stale source. No field/function/body changes or defaults.
- `crates/world-rust/src/lib.rs`: stable switchable rule
  `rust.remedy.derive_debug`, used only when existing compiler suggestions yield
  no edits. No iterative speculative follow-up for these proposals. Every
  requested check must pass, across sources, before offering the repair; existing
  apply verification/rollback remains. Existing suggestion groups/parser are
  unchanged, so this batch does not claim improved nested/multipart handling.
- `crates/world-rust/src/checks.rs`: two existing tests now put their artifacts
  under the configured scratch directory instead of repository `target`.
- `tools/coding-diagnostics-dev.py`: workspace-only replay and independently
  authored functional fixtures. Neither expected patches nor hidden tests are
  consumed. Functional probes are installed after the repair invocation.
- This report. No shared manifests, rename modules, index, CLI, loop, scorers,
  historical prompts or README files changed. No Cargo.lock change.

Final measured results (verified / refused / wrong):

| Scope | Baseline | Candidate |
| --- | --- | --- |
| Fixed sample, 3 supplied prompts | 0 / 3 / 0 | 1 / 2 / 0 |
| All 18 supplied prompts | 0 / 18 / 0 | 3 / 15 / 0 |
| Eight independent fixtures | 0 / 8 / 0 | 4 / 4 / 0 |

The three dev gains are exactly the three request styles for one workspace,
not three independent repairs. Both preregistered functional outputs match
exactly for every gain. The only source edit adds standard Debug to Reading;
its fields, existing Clone/PartialEq derives and warmest implementation remain
unchanged. All other supplied sources remain byte-identical and refused.
Independent record, Unicode tuple, enum and nested module types now print their
predeclared outputs. Ambiguous names and conditional declarations refuse.
Non-Debug fields and unrelated compiler errors fire the new rule but fail
scratch verification, leaving the original source byte-identical.

Validation: final world-rust unit suite **19/19**, including all 16 existing tests
and 3 new tests; focused all-target clippy with warnings denied passes; CLI debug
build passes; cargo fmt and git diff --check pass. Unit coverage includes Unicode
full-item anchoring, existing attributes, duplicate findings, stale-source atomic
refusal, type aliases, ambiguous/conditional/test/custom-derived declarations,
non-identifier type names, unrelated diagnostic/source IDs and out-of-root
symlinks. Final compiler is `rustc 1.99.0 (b940084d7 2026-09-28)`.

Exact new observed errors: **zero** on the supplied18 and independent8. No claim
about full765 errors is possible yet. **Coordinator must run the unchanged
full765 parent gate before accepting/integrating this candidate.** Worker has not
read that evaluator, expected patches, official test contents, hidden tests or
old private logs. Acceptance is therefore local-only/pending parent regression.

H1 was rejected by compiler evidence without a runtime edit. H2 produced a gain;
its initial empty insertion anchor was tightened to the full original declaration
before final replay. Initial own fixture design used a single-file crate, where
rustc already suggests the derive; changed it to independent library/binary
fixtures before scoring them so tests exercise the new rule. No scored failed
experiment was discarded. Initial and final candidate outcomes are identical.

## Provenance and frozen evidence

Worker artifacts: `/home/gsc/.cache/glassmind/continuous-20261010/repairs2-run/tmp`.
Targets: sibling `target`, always the supplied CARGO_TARGET_DIR. All cargo runs
used TMPDIR, systemd user scope MemoryMax=8G/MemorySwapMax=0, setsid --wait,
timeout600/kill-after10, -j1 (except fmt), and test threads4. Fixtures run
sequentially within each replay, with distinct baseline/candidate/fixture targets.
No agents, model calls, services, commits or surviving background jobs.

- Baseline binary: `glassmind-baseline`, source `0bff6a1`, SHA256
  `0227f639013d66fa4befe07f0487590b491b9d30981217f8668821daf895240c`.
  Verified against base `05e80930ef2b06e8e054ece3ab4cc69666d19b98`:
  entire checks.rs (30651 bytes), slips.rs (13463 bytes), and lib.rs remedy/apply
  region (7663 bytes) are byte-identical. This baseline comparison is specifically
  repair-scoped; package renames, ratios and dialogue are outside this experiment.
- Final debug binary SHA256:
  `58b9eb93a84387e133ef0e20f48343e513890e5dec06db291268e40c055cc611`.
- `audit/*.jsonl`: fresh original rustc diagnostic JSON for all six workspaces.
- `baseline-{sample,full,fixtures}/results.json` and
  `candidate-final-{sample,full,fixtures}/results.json`: frozen output, diagnostics,
  exact binary hashes, behavior results and rule traces. Earlier
  `candidate-{sample,full,fixtures}` are pre-anchor revision evidence, not final.
- `handoff-manifest.json`: exact final owned source/report/tool bytes and SHA256,
  plus final result hashes; `dev-input-provenance.json`: every permitted input
  file's bytes/hash. Source snapshot total **4895 bytes**: 19 workspace files
  **3555 bytes**, six prompt files **1340 bytes**. No upstream LICENSE supplied;
  no license inferred, no dataset sources redistributed. Inputs are coordinator
  supplied; prior project evidence pins dev to RustBench
  `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, not independently authenticated here.
- New authored fixtures version `coding-diagnostics-dev-v1`: **1522 bytes** of
  Rust source/manifests across eight crates, plus **318-byte** functional probe.
  License follows repository **MIT OR Apache-2.0**. Dataset downloads **0 bytes**.

Limitations: exact rustc English diagnostic shape; only unqualified nongeneric
names; production discovery inherits existing declared-module traversal limits.
This is Debug derivation only. Copy semantics, inferred collection types,
numeric casts, visibility and missing match behavior remain unsupported. Built-in
spelled existing derives are not a complete macro name resolver. Compiler pass
alone is not semantic proof; provided gains have additional independent checks,
but full regression remains the coordinator's responsibility.

## Reproduction

From this checkout, use fresh output directory names on reruns:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/repairs2-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/repairs2-run/target
export GLASSMIND_SCRATCH="$TMPDIR/repro-unit-scratch"
capped() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
capped cargo fmt --all -- --check
capped cargo test -p glassmind-world-rust -j1 -- --test-threads 4
capped cargo clippy -p glassmind-world-rust -j1 --all-targets -- -D warnings
capped cargo build -p glassmind -j1
capped python3 tools/coding-diagnostics-dev.py --binary "$TMPDIR/glassmind-baseline" --dev-inputs ../repairs2-run/dev-inputs --sample --output "$TMPDIR/repro-baseline-sample"
capped python3 tools/coding-diagnostics-dev.py --binary "$TMPDIR/glassmind-baseline" --dev-inputs ../repairs2-run/dev-inputs --output "$TMPDIR/repro-baseline-full"
capped python3 tools/coding-diagnostics-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs ../repairs2-run/dev-inputs --sample --output "$TMPDIR/repro-candidate-sample"
capped python3 tools/coding-diagnostics-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs ../repairs2-run/dev-inputs --output "$TMPDIR/repro-candidate-full"
capped python3 tools/coding-diagnostics-dev.py --binary "$TMPDIR/glassmind-baseline" --output "$TMPDIR/repro-baseline-fixtures"
capped python3 tools/coding-diagnostics-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/repro-candidate-fixtures"
```


Coordinator review: direct item attributes and inline module ancestry reject
conditional/unknown attributes. Out-of-line discovery uses the existing production
module walker, which excludes test-only modules and custom paths/cfg_attr but can
follow other conditional module declarations. This is not a claim of resolving
all Cargo configurations. The complete requested check must still pass on the
current build configuration before any new derive is offered. Detailed compiler
logs remain outside git; summarized worker evidence and hashes are in
`coding-diagnostics-evidence/`.


Coordinator full development gate: source2ce9230 on RustBench f79e71a,
all765 requests, improves163pass599refused3fail to166pass596refused3fail.
Only fix-errors/derive-debug changes, for all three request styles; every other
outcome is identical. Frozen binary/result hashes and each transition are in
[the comparison](results/coding-standard-debug.json). No held-out evaluation.
