# Bounded Cargo package renames

Preregistered before implementation, base fcb81dbf17273dcca4d45f7a8df4d3428fb217a0.
Hypothesis: root-specific symbol identities plus bounded explicit import/reexport
resolution allow safe renames across one package's library and binary.
Acceptance: all fixed positives make exact AST identifier edits, preserve comments,
strings and runtime output; negatives preserve every source byte. Retain types9/9
and variants14/14. Run sample before full local set, then all six permitted dev
prompts on fresh copies with cargo checks. No benchmark correctness claim.

Fixed independently authored cases (implementation must not recognize their names):
1. Hyphenated package, custom library name, Unicode struct, chained reexport,
   alias named Local independently in both roots, literal comments/strings.
2. Enum variant through reexported enum and explicit variant alias across roots.
3. Standard Clone/Copy/PartialEq/Eq/Default derives and literal documentation.
Negatives: target Debug derive (observable spelling), custom derive, cfg,
unknown macro, cyclic reexports, alias shadow, unresolved external import,
workspace members, unaccounted source, duplicate root declarations, and compiler
failure requiring transactional rollback. Positive sample is cases1–3.

Debug policy: refuse Debug on the renamed struct/enum or the owner of a renamed
variant. Generated Debug strings can expose the old spelling, even through public
APIs not exercised by a local run. No claim of unchanged general behavior from a
build or sample output. Custom/procedural derives and nonliteral doc attributes
remain unsupported. Literal docs remain byte-for-byte unchanged.

## Implementation boundary

`rename_package.rs` parses Cargo's manifest with `toml`, identifies library and
binary roots independently, and assigns declaration identities by target root
and module path. It handles an explicit `[lib].name`, default package-name
hyphen normalization, conventional `src/lib.rs`/`src/main.rs`, local imports,
renamed imports and chained reexports. Every import must resolve, including
unused imports. Deterministic ordered maps reject collisions; cycle tracking
and a 64-hop bound reject cyclic or excessive chains. Source discovery is capped
at 4096 entries and module nesting at 64. No cargo metadata process or server is
needed for resolution.

Both rename visitors use this graph and retain their exact AST-span accounting:
every old-name identifier must have a proven binding; comments and string
literals are unchanged. A file compiled in both targets refuses because one
physical span would represent multiple identities. Multiple loaded sources,
workspace/member/target ambiguity, nested manifests, symlinks, unaccounted Rust
files, cfg, glob imports, opaque aliases, raw identifiers, shadowed bindings,
unknown macros and unresolved external imports refuse. New-name collisions
remain a global conservative refusal within the source package. Conventional
single-file inputs without manifests retain prior local behavior.

Supported derives are unqualified builtin Clone, Copy, PartialEq, Eq, PartialOrd,
Ord, Hash and Default. Imports shadowing these macro names refuse. **All Debug
derives refuse**, a deliberately stronger condition than the preregistered
minimum target/owner check. This avoids having to establish that a derived
container cannot transitively expose the renamed type/variant spelling.
Literal prose documentation is permitted; fenced/indented doctests, intra-doc
links and nonliteral doc expressions refuse. Output macros remain opaque to
edits; nested `!` tokens refuse (also conservatively rejecting unary negation
inside those macros). No custom derive or cfg expansion was added.

Compiler verification and transactional application use the existing integrated
CLI; no CLI, English, scorer, loop or runtime adapter changes were needed.
Successful compilation does not establish arbitrary behavioral equivalence;
public type reflection and external consumers remain outside this closed package
scope. This batch makes no benchmark correctness or full765 improvement claim.

## Rejected attempts and provenance

- The first baseline build overlapped source editing and failed before producing
  a usable binary. Base source was restored and the baseline rebuilt successfully
  before being copied to TMPDIR/glassmind-baseline; the failed build is not used.
- Candidate1 used the TOML value parser rather than document deserialization.
  It refused all three positive fixtures. A focused graph test isolated this;
  switching to `toml::from_str` restored the intended manifest interpretation.
- Blanket allowance of Debug/cfg/unknown macros was rejected on semantic grounds;
  no benchmark-directed relaxation was attempted.
- The fixture designs and acceptance criteria above were recorded before Rust
  implementation. Their executable literal source pairs were written during
  implementation, before the first baseline/candidate evaluation. Supplemental
  unit checks were then added for the observed parser defect, root aliases,
  default library naming and non-inert docs; they do not change the fixed15 set.

The fixtures are independently authored here, under this repository's
MIT OR Apache-2.0 license; no downloaded dataset or expected benchmark patch is
embedded. The two coordinator-supplied workspace-only dev snapshots and their prompts
are used only by the development harness, never runtime. Their upstream license
was not included in the supplied copies; no new upstream attribution is inferred.
Hashes and byte counts for the exact supplied sources/prompts, authored fixtures,
and new dependency archives are recorded in the run's provenance.json.

## Final evidence (uncommitted batch)

Base: `fcb81dbf17273dcca4d45f7a8df4d3428fb217a0`. No commits were created.
Artifacts: `/home/gsc/.cache/glassmind/continuous-20261010/packages-run/tmp`.
`handoff.json` records exact source hashes, binary hashes and totals;
`provenance.json` records all source/dependency hashes, licenses and bytes.

| Fixed evaluation | Baseline | Final |
| --- | ---: | ---: |
| Package positive sample | 0/3 | 3/3 |
| Full independent package set (3 positives, 12 safety/rollback negatives) | 12/15 | 15/15 |
| Prior type fixtures | 9/9 | 9/9 |
| Prior variant fixtures | 14/14 | 14/14 |

All three package edits match literal expected sources exactly and retain the
same stdout on before/after cargo runs. All twelve negatives preserve source
bytes. The rollback case reaches a three-span proposal, fails compiler
verification, and leaves both roots unchanged. Several negative fixtures are
intentionally invalid Rust and establish refusal/rollback, not behavior.
Nine focused world-rust rename unit tests pass; world-rust fmt and library
clippy with `-D warnings` pass; the main integrated release CLI builds.
No workspace-wide giant suite or full765 evaluation was run.

Final binary SHA256:
`8da34939a7de111626740405f0d99a57c9ce5586586d2abe07151dd2e3fb388f`.
Baseline binary SHA256:
`13616ad1f953bd6a3dde53155011d52ac49fd209fbe2bd794bba5c558018ebb6`.
Both binaries are retained as `glassmind-candidate-final` and
`glassmind-baseline` in TMPDIR.

All six supplied prompts were run unchanged on fresh copies, with cargo
`check --all-targets` before and after. All twelve checks passed; **zero edits**
were made, and every CLI reply matches the baseline. Struct terse and variant
terse/hurried reach unsupported rename replies. Struct chatty/hurried fail
parsing; variant chatty takes the existing fix-tests fallback and declines.
Thus the raw prompts do not all parse as renames in this integrated CLI.
The supplied source also contains cfg-test modules, Debug derives, and unknown
macros; support for those remains intentionally outside this batch. This is a
measured local capability gain, **not a supplied-dev or benchmark gain**.
The coordinator must run unchanged full765 before shipping.

Source data: authored fixture version `fixed15-v1`, MIT OR Apache-2.0,
3816 bytes of generated input files including each Cargo manifest (2486 bytes
of literal before-payloads as stored in the harness). Supplied
workspace sources plus their six prompts total 6932 bytes; upstream revision
and license were not included, so provenance is by exact file hashes.
The new parser dependency is toml `0.9.12+spec-1.1.0`, MIT OR Apache-2.0,
56485 bytes for its registry archive. All ten new lockfile dependency archives
(including optional packages) total 782407 bytes; individual versions, license
strings, bytes and checksums are in provenance.json. No dataset was downloaded.

Extra-target declarations, target-specific dependency tables and `.cargo`
configuration also refuse; they can change which roots or extern names Cargo
compiles. A focused test verifies refusal when an example reuses the library
source. This conservative boundary is deliberately narrower than general Cargo.
The graph does not provide name resolution for workspaces, dependency APIs,
custom paths/targets, type aliases as variant owners, or arbitrary macro syntax.

## Reproduction

From this worktree, use fresh output directories. The legacy runner retained in
TMPDIR executes the existing unchanged harness definitions, modifying only the
resource environment to give each fixture its own build/cache namespace. Every
harness runs sequentially; no evaluation definitions were modified.

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/packages-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/packages-run/target
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build --release -p glassmind -j1
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind-world-rust rename_ -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy -p glassmind-world-rust --lib -j1 -- -D warnings
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt -p glassmind-world-rust -- --check
python tools/coding-packages-dev.py --binary "$TMPDIR/glassmind-candidate-final" --sample --output "$TMPDIR/repro-package-sample"
python tools/coding-packages-dev.py --binary "$TMPDIR/glassmind-candidate-final" --output "$TMPDIR/repro-package-full"
python "$TMPDIR/run-legacy.py" types "$TMPDIR/glassmind-candidate-final" "$TMPDIR/repro-types"
python "$TMPDIR/run-legacy.py" variants "$TMPDIR/glassmind-candidate-final" "$TMPDIR/repro-variants"
python tools/coding-packages-dev.py --binary "$TMPDIR/glassmind-candidate-final" --dev-inputs "$TMPDIR/../dev-inputs" --output "$TMPDIR/repro-supplied-dev"
```

Repeat the harness commands with `glassmind-baseline` and different output
names for the preserved baseline. The outer worker scope was verified at
8589934592 bytes MemoryMax and zero MemorySwapMax. All cargo invocations used
the required scopes, process groups, timeout, TMPDIR and single-job setting;
focused tests used four threads. No agents, model calls or persistent services
were launched. Token accounting for this worker is left to the coordinator.

Coordinator provenance: both workspace-only snapshots and prompts came from RustBench revision `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`. No upstream LICENSE file was present; no license is inferred. Raw workspace source snapshots remain outside git. Worker handoff/provenance and local result JSON are retained in `coding-packages-evidence/`.


## Coordinator integration

Source `05e80930ef2b06e8e054ece3ab4cc69666d19b98`. Independent integrated CLI
runs pass sample3/3, full15/15, type9/9 and variant14/14. The same six supplied
dev prompts still edit nothing, with all before/after builds passing. Workspace
fmt/all-target clippy/release/strict4/4 smoke and all16 world-rust tests pass.
Reports and binary hashes are in `coding-packages-evidence/integrated/`.
Full765 RustBench dev is complete:163 pass,599 refused,3 fail, with every
(task,style) outcome unchanged. This is a local package capability, not a
benchmark gain. Exact comparison: `results/coding-package-renames.json`.

A separate coordinator probe tested a target-specific dependency colliding with
the library's crate name: rustc rejected the original program with E0464. It is
not evidence of a successful rename or a semantic regression. The final worker
already rejects target-specific dependency tables and extra target declarations.
