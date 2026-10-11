# Extremum initializer batch

Outcome: public TRAIN 7 verified / 20 refused -> **8 verified / 19 refused**.
No failed native verifications. All seven previous patch records and complete
response objects are exactly preserved. Fixed sample unchanged: 2 verified / 2
refused. Both full probes select the same 27 conversations, input SHA256
`6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164`.

New TRAIN repair: winter-high src/lib.rs:13, column 19, `0` -> `<i32>::MIN`.
This is one exact initializer replacement; no other source/test byte changes.
Public native verification is evidence, not proof of intended semantics.

## Exact implementation

- New crates/world-rust/src/extremum.rs resolves signed integer slice/Vec input
  types or directly declared struct Vec fields in a closed root namespace.
  Supports all six signed widths, strict min/max comparisons in either operand
  order, immutable shared-reference parameter, reference element pattern,
  literal (including negative or suffixed) seed, optional exact accumulator type.
- A four-statement body proves the dominating same-collection empty => None
  guard, isolated mutable accumulator, one pure conditional assignment per
  iteration, and unmodified Some(accumulator) result. Extra statements, effects,
  collection mutations, shadowing, alias/custom types, generic code, attributes,
  macros, imports, modules, custom iterators, extra folds and unknown flow refuse.
- checks.rs retains the original legacy candidate vector, budgets and native
  scalar-witness logic in undo_legacy_slip. Public slips appends extrema after
  all existing families. Legacy accepted or ambiguity-withdrawn choices cannot
  fall through. Exhausted legacy time also prevents fallthrough. Only otherwise
  do up to eight new candidates use a separate 20-second native-check budget.
  Any incomplete verification or two passing candidates refuses. New choice
  evidence must be saved before acceptance. No loop lowering or invented witness.
- New native tests and new tools/coding-extremum adapters/reports. No English,
  CLI, loop, scorer, shared manifest, other crate or existing test edits.

## Validation

- 51 world-rust tests passed (39 unit + 12 integration); expanded extremum-only
  rerun also passed after adding the stronger full-boundary ambiguity contract.
- 26 frozen/revised native contracts compile and execute failing tests initially:
  12 positive (all widths x both extrema), 12 refusal controls, 2 competing cases.
  Two additional native Vec/reversed-comparison/typed-negative-seed cases pass.
  Tests cover empty, MIN/MAX, zero, all-negative/all-positive and mixed inputs,
  direct slices, struct fields, Unicode names and exact byte preservation.
- Each competing case separately native-tests both alternatives unchanged, then
  runs each example: `Some(-2) Some(0)` versus `Some(0) Some(-2)`. Family verifier
  withdraws both; the stronger case also proves end-to-end withdrawal. Original
  weak competing case exercises only the family verifier because legacy proposals
  may satisfy its weaker public assertion. No semantic witness is claimed.
- Incomplete >8-candidate refusal and strict old-vector-prefix checks pass.
- Original argument-swap worker25 + parent13 contracts: **38/38 exact**, all
  originals compile and genuinely fail executed tests, all 20 offered repairs
  pass native tests. Original expectation files were not edited. Two old
  competing-swap choice records are retained (both alternatives passed).
- fmt check, all-target world-rust clippy with -D warnings, release build pass.
  Final no-op release rebuild matches frozen binary exactly.

## Provenance and reproducibility

Base source HEAD `a716079f6713e190fe543d7ecf6599ce7c14369d`, uncommitted changes.
Baseline binary SHA256
`17b6ceb211602871b22da3afd15c05af8ba409595a220c379afde91ddf540b63`.
Frozen candidate binary SHA256
`a6e9d59a9a62fe0216304bfdd43795b2c5dffda9f0777db6162260e047a62b98`.
Exact source hashes, original files, patch, commands, reports and native outputs
are under RUN/frozen-candidate with manifest.json. RUN is
/home/gsc/.cache/glassmind/continuous-20261010/extremum-run.
See COMMANDS.md for reproduction; unchanged tools/coding-repair-dev.py was used.

No new external data or downloads (0 bytes). Existing mounted public TRAIN source:
rustbench f79e71a, MIT OR Apache-2.0, 110 manifest-listed files / 57,265 bytes,
all sizes and SHA256 values verified. Filtered source has 81 requests / 27 sources;
labels contain only kind/split. New authored contracts follow repository terms;
exact byte sizes and hashes are recorded in the frozen manifest. No credentials,
API keys, model calls, agents, hidden/test-split assets or private logs were read.

## Retained failures and limits

Original pre-edit PLAN/contracts.json are retained. One custom-Deref negative
fixture initially failed compilation: private tuple field and unsupported implicit
IntoIterator assumption. contracts-native.json makes the constructor public and
uses explicit &**xs; assertion and refusal expectation unchanged. Initial test log
and fixtures retained. Static review rejected the planned ::core path in favor of
<primitive>::MIN/MAX to avoid external-crate namespace assumptions. See ATTEMPTS.

Scope intentionally excludes inline test modules, methods, iterator adapters,
unsigned integers, multiple parameters, other empty policies and broader flow.
The native acceptance search is bounded and public tests may underconstrain intent.
Full 765 DEV and independent competing-repair contracts were NOT inspected or run;
coordinator owns those gates (all 206 current passes retained / zero failures).
No claim of full DEV acceptance or broad semantic correctness is made.

Every explicit cargo invocation used persistent TMPDIR, an 8GiB/0swap systemd
scope, setsid --wait, 600-second timeout and -j1 (fmt excepted); tests use four
threads. New authored builds use the existing CARGO_TARGET_DIR. Existing unchanged
isolation regressions intentionally exercise disposable scratch target behavior;
that legacy behavior was preserved. Outer worker scope is also 8GiB/0swap.
No commit, services, downloads or owned jobs remain running at handoff.
