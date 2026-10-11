# Independent field-owner review

Result: **no concrete unsafe accepted edit found in this bounded review**. The
candidate satisfies all 21 independently authored contracts: six exact edits
(five newly supported, one inherited) and fifteen byte-preserving refusals.
This supports the narrow implemented subset; it is not a proof for arbitrary
Rust or a replacement for the coordinator's pending full765 run.

Only this new review directory is left uncommitted. No implementation, original
test/evaluator, submitted archive, binary, shared manifest, or historical prompt
was changed. The exact archived runtime patch was temporarily applied solely
for source identity verification and then reversed. No commits, agents, model
calls, credentials, downloads, services, or surviving jobs.

## Identity verified before probes

Review HEAD: `47b6b285b5cd23f0580600dcda3011689ab20da1`.
Archived source base: `5f2859a730f6d20a0ac64ca9a7551821ed123e86`.
`git diff` between these commits for `crates/world-rust` is empty.
The exact archived `runtime.patch` applied cleanly; `cmp` matched **both**
resulting runtime files to the immutable archive. Reversing it restored an
empty tracked diff. These source hashes also match the archived artifact manifest.

| Artifact | Bytes | SHA256 |
|---|---:|---|
| Supplied `glassmind-accepted` | 127487680 | `19d2770f8a43cdd9037b8a46747d5a428489b567dcdfd63e40e1bc1e011fb621` |
| Supplied baseline `glassmind` | 74458648 | `ca120ec676a8979d9116ef10380e58cd6f7f3d033fc97ade968a7d510c5b1bd7` |
| Archived `runtime.patch` | 14319 | `7b7806e1f8d2c301aee3b9a058770f5ff90c88e7e9ad7d962d8b060b5ad1765d` |
| Archived `world-rust/src/lib.rs` | 49407 | `95e0a1a5e0cb81dc1c20d4c1ab16b522eb7cf602d955dd1a9453e4c33d6a646e` |
| Archived `rename_fields.rs` | 10818 | `524aa731f0a7624d3a7ad1702c475463e1edd6433d515c75ce8940b0ea3a9921` |

Only these two supplied executables were attempted. No intermediate candidate
was used and no replacement executable was built. Binary identity establishes
the exact supplied bytes; source-to-binary provenance additionally relies on
the supplied archive attestation, not a claimed reproducible rebuild.
Full paths and environment versions are in [identity.json](identity.json).
Fixture compiler: `rustc 1.99.0 (b940084d7 2026-09-28)`; Python 3.14.7.

## Argument audit

Line numbers below refer to the immutable submitted source under
`/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run/submitted/`;
the unchanged helper files refer to review HEAD.

- **Target identity and routing:** archived `lib.rs:589–641` requires a
  homonymous field, one explicitly resolved struct owner, matching source and
  owner name, and one source package. `rename_fields.rs:28–62` additionally
  requires one struct of the selected owner name across the package, then
  matches its field declaration by file and line. Duplicate owner names refuse,
  even when the second struct has no old-name field (`duplicate_identity`).
- **Complete accounting:** `rename_fields.rs:63–90` seeds all old-name struct
  declarations, then requires equality between classified positions and the
  index's complete old-spelling token set. The edit set contains only selected
  owner positions. `index.rs:128–147` builds that token set from source tokens;
  `rename_types.rs:23–26` uses matching file/line/column coordinates. Exact
  before/after checks exercised Unicode preceding edits and retained CRLF bytes.
  Raw field spelling refuses at `rename_fields.rs:18–23`.
- **Paths and receiver identity:** `rename_fields.rs:106–131` accepts resolved
  struct paths without path arguments or qualified-self syntax, explicit
  parameter types (including references), direct literals and transparent
  parentheses/references. `rename_package.rs:364–432` resolves lexical
  `crate/self/super` and library-qualified paths. Independent successful probes
  cover `super`/`crate` across module files and library paths from a binary target.
  Arbitrary member chains refuse (`qualified_member`), rather than being
  mistaken for direct typed receivers.
- **Bindings/scopes:** `rename_fields.rs:158–181` saves/restores parameter maps;
  `196–213` rejects rebinding and block-local items. Match and for-loop shadowing
  both refuse. Imports, type aliases, generics, impls, traits and closures are
  explicitly excluded (`154–156`, `203–225`). `raw_receiver_binding` is a valid
  Rust spelling contrast that conservatively refuses; it is not an unsafe edit.
- **Literals and uncovered tokens:** `182–195` recursively visits nested
  expressions and classifies only explicit literal members. Nested opposing
  owners and struct-update bases produced exact edits. Destructuring and enum
  variant field declarations leave uncovered old-name positions and refuse.
  Set equality establishes coverage only after these owner-resolution checks;
  equality alone would not establish correct ownership.
- **Attributes/macros/package closure:** `227–238` excludes unknown macros and
  non-inert attributes. `rename_package.rs:9–68` narrowly defines inert attributes
  and output macros; old tokens inside even a permitted output macro remain
  unclassified. `load_mode:86–247` and `rename_types.rs:42–90` reject uncovered
  sources, ambiguous target coverage and unsupported module attributes. Probes
  independently refused macro tokens, cfg, an unrelated `allow` attribute, and
  a Rust file outside the module graph. Native compilation of every original
  rules out passing these refusals merely because the fixtures were invalid.
- **Collision and fallback:** `rename_fields.rs:18–23` rejects any existing new
  spelling, including raw spelling. `lib.rs:640` returns the new route's failure;
  `846–848` propagates it without entering the generic fallback. The original
  `rename_at` homonym guard remains at `669`. The unique-spelling control edited
  exactly on both binaries. No inherited defect was reproduced; this one
  control does not establish all inherited behavior is correct.

## Frozen independent measurements

Before creating fixtures, the stated hypothesis was that coverage can be
complete yet ownership wrong. Acceptance required byte-exact owner-specific
edits and unchanged behavior of both owners, or byte-identical conservative
refusal. Expectations were frozen before **each** suite's first probe.
The supplemental suite was authored after the first suite's results, not
represented as part of the initial frozen sample.

| Suite | Baseline edits / refusals | Accepted edits / refusals | Native original/after behavior |
|---|---:|---:|---|
| Fixed first-three sample | 0 / 3 | 3 / 0 | all pass on both binaries |
| Full primary, 15 cases (includes sample) | 1 / 14 | 5 / 10 | all pass on both binaries |
| Supplemental, 6 cases | 0 / 6 | 1 / 5 | all pass on both binaries |

The 21 distinct final contracts therefore change from 16 satisfied / 5
unsupported-positive refusals to 21 satisfied. All baseline expected refusals
also matched their frozen baseline expectations; these are not baseline edits
or coverage gains. The sample is a subset and is not added to the denominator.

New exact edits: `nested_literal`, `qualified_files`, `struct_update`,
`unicode_crlf`, `lib_bin_paths`. Inherited exact edit: `inherited_unique`.
Unchanged refusals: `match_shadow`, `qualified_member`, `duplicate_identity`,
`macro_token`, `cfg_attribute`, `uncovered_source`, `new_collision`,
`raw_spelling`, `alias_parameter`, `generic_parameter`, `for_shadow`,
`raw_receiver_binding`, `destructure_member`, `enum_variant_collision`,
`unrelated_attribute`.

Every fixture compiled before invoking glassmind. Original tests compare the
two owners' observable values to the frozen `(13, 71)` pair and emit a named
marker. The cross-target case runs these checks in both library and binary.
The same original assertions passed afterward; exact source comparisons cover
every supplied `.rs` file and preserve tests, unrelated fields and strings.
Manifests remained unchanged. Before/expected/after files, complete CLI output,
and native test logs are retained beside each `result.json` in `reports/` and
`supplemental/reports/`; [summary.json](summary.json) contains aggregate counts.
Successful source diffs were also manually inspected.

**Rejected experiments / failed assumptions:** no original fixture failed
compilation or its behavior check; no expectations were relabeled; no candidate
repair was attempted. The attempted counterexample hypotheses (misassigned
nested owners, path confusion, shadowing, incomplete token accounting) did not
produce an unsafe edit. The raw-parameter spelling refusal was preregistered.
After all probes, only an optional `--tag` replay-output suffix was added to
the new runners; frozen inputs, expectations, executed command bodies and
result records were unchanged.

## Provenance and reproduction

New data is independently locally authored visible development Rust, version 1,
licensed **MIT OR Apache-2.0**. No archived generator was imported. Primary
inputs total **6,025 UTF-8 bytes**, expected outputs **6,059**; supplemental
inputs **2,629**, expected outputs **2,639**. Combined: **8,654 input bytes**,
**8,698 expected bytes**, 21 cases. Downloads: **0 bytes**. Frozen JSON hashes:

- Primary: `1a4ea02704be95b44d718ce55a8df768f4c27c76499596e6e8614ab19ef3bbcf`.
- Supplemental: `f40ccac26e40a8fe4675879af0d7405bd8a154d78b915a8c370e6b5d77f20490`.

From this checkout (choose a new tag on each replay; existing artifacts are
deliberately never overwritten):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/fieldreview-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/fieldreview-run/target
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/fieldreview-run/scratch
python tools/coding-field-owners-review/probe.py --freeze
python tools/coding-field-owners-review/probe.py --binary baseline --sample --tag replay1
python tools/coding-field-owners-review/probe.py --binary accepted --sample --tag replay1
python tools/coding-field-owners-review/probe.py --binary baseline --tag replay1
python tools/coding-field-owners-review/probe.py --binary accepted --tag replay1
python tools/coding-field-owners-review/supplement.py --freeze
python tools/coding-field-owners-review/supplement.py --binary baseline --tag replay1
python tools/coding-field-owners-review/supplement.py --binary accepted --tag replay1
```

Both runners assert the supplied binary hashes. Every cargo/CLI mutation runs
under `systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait
timeout --kill-after=10s 600`, with persistent TMPDIR and explicit scratch.
Fixture cargo uses `test --offline -j1 -- --test-threads 4 --nocapture`;
inherited native checks also enforce scopes, persistent TMPDIR, `-j 1` and
four test threads. The existing target directory is reused. The outer review
scope was independently verified as 8589934592-byte memory / zero swap.

Limitations: no full765, existing full-regression rerun, candidate rebuild,
workspace suite, release build or CI result is claimed. Arbitrary Rust,
external dependencies and compiler name-resolution equivalence remain outside
this evidence. The coordinator should retain the original candidate and run
its required independent full765 before publication. All runtime sources are
restored and all review subprocesses completed.
