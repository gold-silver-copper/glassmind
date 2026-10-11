# Speech-scope repair veto — candidate pending parent gates

One uncommitted batch based on `6a346c3080a5dd94d7a6bc7dae32af7f6d969798`.
Only `crates/cli/src/coding.rs` and new `tools/coding-repair-veto` are changed.
No agents, model calls, downloads, services, credentials, commits, new model
weights, runtime sentence tables, evaluation definitions or parent independent
cases were used. No backend/world/mind/edit/mainCLI/shared-manifest changes.
The 2,344-byte existing rename-routing body is byte-identical.

## Implementation

A bounded (8,192 bytes /512 visible tokens) quote-aware clause analyzer preserves
legacy positive inference and removes Repair readings for explicit non-authorizing
speech acts. It distinguishes status from repairs, quoted data from outer speech,
requested-action negation from failure evidence, and explanatory `get why/how/what`
from a repaired object/result. It examines later clauses for retraction, conditions,
deferred authorization and competing operations. Complete supported status questions
become existing Ask(Passes/Problems); matching legacy readings are retained exactly.
Unknown status tails cannot supply permission. The same filter covers understand,
guesses and learned coding fallback. The old fallback's blanket `without` rejection
is replaced by the scope analysis, permitting behavior-preservation constraints.

This is **not accepted**: parent full765 must preserve all original180 passes,
introduce no new failed outcomes, and pass the separately frozen independent
explicit probes including8 positives and9 historical unauthorized native edits.
None of those parent gates were run or their case contents read by this worker.
Reject if any fail; do not reinterpret labels or weaken the score.

## Validation

Final results are recorded in `typing-summary.json`, every row in
`typing-outcomes.tsv`, and actual mutation/native checks in `native-summary.json`.
All original68 labels and bytes are unchanged. The preregistered32 and later
supplemental9 are separate groups; supplemental probes are not independent
preimplementation evidence. The TRAIN request-marker annotation is heuristic,
not authorization truth. Full results and final counts are appended below.

All21 CLI tests (including existing rename and Remedy-confirmation contracts),
all40 English/Rust tests, all-target package clippy with warnings denied, fmt
check, and release build pass. Shell syntax and Python compilation checks pass.
Exact source and binary were frozen before final native evaluation; no further
runtime edits followed. Native fixtures have unique package names and unset
CARGO_TARGET_DIR, retain the original assertion `number()==7`, and permit only
`7u8`→`7u32`. Every initial ask --apply stays read-only; explicit `yes --apply`
confirmation is checked separately for build and test.

The old erroneous initial-apply expectation is not silently relabelled: all24
old68 positive initial apply calls and all15 legacy positive initial apply calls
remain unchanged, thus fail that old expectation. This batch measures ask --apply
and do, plus two plain-ask/yes confirmations; it does not claim the original
204-execution three-mode suite was rerun or passed.

## Reproduction and evidence

Persistent run root:
`/home/gsc/.cache/glassmind/continuous-20261010/repairveto-run`.
Final release: `frozen-final/glassmind`; final source: `frozen-final/coding.rs`.
Final native logs/results: `final-native-sample`, `final-native-full`,
`final-supplemental-native`; typing: `baseline-optimized.json`,
`final-optimized.json`, `supplemental-baseline.json`,
`final-supplemental-typing.json`. All raw logs stay outside Git.
`HASHES.json` pins source, binary, input, probe, and raw result bytes.
`EXPERIMENTS.md` records rejected intermediate candidates and stopped debug runs.

Reproduce from the final uncommitted source:

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/repairveto-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/repairveto-run/target
export REPAIR_RUN=/path/to/fresh/owned/run
# Copy the supplied baseline/, oldcases/, training-requests.tsv and provenance
# into REPAIR_RUN, preserving the hashes in HASHES.json.
bash tools/coding-repair-veto/reproduce.sh
```

The script includes exact fmt/test/clippy/release, optimized baseline/candidate
typing, fixed native sample, full115 native, and supplemental commands. Every
cargo invocation uses persistent TMPDIR, the existing repository target,
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, `-j1` except fmt, and test threads4. External native fixtures
unset the repository target and use their own targets. Outer worker scope was
verified MemoryMax8589934592/MemorySwapMax0. No jobs are intentionally left running.

## Sources and limits

No external data was acquired. New locally authored contrasts use the repository
MIT OR Apache-2.0 license; version is each file's SHA256 in HASHES.json.
Original68:2,889 bytes; legacy15:1,065 bytes; supplied public TRAIN172:14,110 bytes,
91build/81test. TRAIN source is public `train-v3/inputs.jsonl`, SHA256
`461fa4b0a7157185b231da0aece444342470f84cad8fb3e151845204cc46602f`, licensed
rustbench MIT OR Apache-2.0 according to supplied provenance. Only text and kind
were supplied/read; no gold patches, solutions, heldout or hidden generators.
An upstream revision beyond this source hash was not supplied. New source bytes
and all request-file hashes are listed in HASHES.json.

Implicit failure reports and underspecified repair requests retain legacy behavior.
Finite grammatical vocabulary is not a general English authorization proof; this
batch claims measured explicit-scope improvements only. All original expectations,
remaining failures and newly gained TRAIN coverage are reported, not relabelled.

## Final measured outcomes

| Typed group | Baseline | Final | Baseline successes lost |
|---|---:|---:|---:|
| Frozen authored32 |14/32|32/32|0|
| Original old68 |34/68|63/68|0|
| Public legacy15 |15/15|15/15|0|
| TRAIN build91 |83/91|87/91|0|
| TRAIN test81 |76/81|80/81|0|
| TRAIN total172 |159/172|167/172|0|

Typed success requires the requested repair kind in understand; status requires
correct Passes/Problems in understand and no Repair in understand or guesses;
refusal expectations require no Repair in either path. The earlier historical
preview-only metric differs: this stricter status/refusal metric also inspects
guesses. Both sides of this table use the same adapter and immutable labels.
The heuristic TRAIN form column has159 overt-marker and13 implicit/unresolved
rows; these are not independently annotated speech acts. All172 outcomes,
including guesses and the five remaining typing failures, are retained in TSV.

Old68 breakdown: positive23/24→23/24; status2/12→12/12; negative9/32→28/32.
Exact remaining typed failures, using zero-based original row indices:

- 10: `I'd like you to repair the build.` — unchanged positive refusal.
- 54: `Could you fix something?` — legacy Repair(Build).
- 55: `Please fix it.` — legacy Repair(Build).
- 56: `The tests are failing.` — Problems(Test), but Repair(Test) in guesses.
- 57: `The build is broken.` — legacy Repair(Build).

Native fixed sample:3/9→9/9; six baseline unrequested edits→zero.
This local nine-case sample is **not** the coordinator's nine historical probes.
Final full115: authored32/32, old68 64/68, legacy15 15/15, total111/115.
All44 of45 expected positive do repairs succeed exactly (only old row10 refuses).
Old rows54/55/57 still edit and are recorded as unrequested under their unchanged
labels. Old row56 remains read-only in actual do. No authored/status/clearly scoped
negative edits. All115 previews preserve source; all115 native before checks fail;
all successful repairs pass the original assertion. Source inventory in each
fixture contains only src/lib.rs, and manifests remain unchanged.

Final supplemental9:9/9 typed and9/9 native, all six negatives unmodified and
all three positives repaired. The previously lost cognitive-get positive is
restored. Explicit confirmation:2/2; source unchanged until yes --apply.

Final source SHA256:
`c5babd57819c68027d9229c5d34c9e3ec1f9fd7cccbf491250850429ff7713f2`
(40,812 bytes). Final release SHA256:
`e4402233b4b1977377f5a8881875444d6fbea17a468ddfa192f346676cb4d532`
(77,811,976 bytes). Supplied baseline release SHA256:
`981f8b100cd82ec9edfa23f2c2d379f2dfb17976e6b32ebab2c68d335c6efdb7`
(77,713,912 bytes). New authored data:contrasts1,652 bytes and supplemental470
bytes, total2,122; no new external data or weights. Final logs are
`final-source-tests.log`, `final-source-clippy.log`, `final-source-release.log`,
`final-fmt-check.log`. Final full typing completed in144.65 seconds.

Status: retained uncommitted review candidate, **not parent-accepted**. No rollback
of final runtime performed; coordinator must run unchanged full765 and independent
gates before integration. All worker-owned child jobs have completed or were
explicitly stopped; no agents or services were launched.
