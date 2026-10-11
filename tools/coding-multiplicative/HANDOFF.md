# Multiplicative repair: bounded implementation handoff

Local authored capability gate PASSED; integration/release is the parent's decision,
subject to full765 regression evaluation. No benchmark gain is claimed. Uncommitted.

## Exact change and frozen contract

Base source commit: `c9d10dc1a1348b757be3ea96db3ee600674e155d`.
Only production edits are five added lines in `crates/world-rust/src/{slips,checks}.rs`:
AST `BinOp::Mul` proposes `/`, `BinOp::Div` proposes `*`; stable sorting of the
complete collected list puts both new kinds after EVERY old candidate across
files and lines. Old relative ordering is identical. No main CLI, manifests,
scorers, benchmark definitions, other crates, historical prompts or README edits.

No changes to ambiguity.rs, native witness execution, or search/resource bounds
(120s search; follow-up at most 64 candidates, 8 native trials, 10s extra).
Passing tests still is not semantic proof. Unsupported ambiguity analysis retains
old behavior. New candidates can withdraw an old patch only through the unchanged
native disagreement guard. Production syntax traversal still excludes explicit
test code, macro tokens, strings, comments, dereferences, types and assignment ops.

`PLAN.md`, `author.py`, `fixtures.json` were hashed before production edits.
`SUPPLEMENT.md`/`supplement.json` were separately frozen after the five-line patch,
before candidate execution; they do not inflate the original eight-positive gate.
Both hash manifests verify. No fixture expectation was changed after measurement.

## Native results

| Scope | Original main | Candidate |
|---|---:|---:|
| Frozen authored positive repairs, exact tree + native assertions | 0/8 | 8/8 |
| Frozen negative crates, unchanged | 6/6 | 6/6 |
| Old cross-file first repair preserved | 1/1 | 1/1 |
| Supplemental compile-failure-before-valid-repair | refused | exact repair |
| Supplemental variable-zero-divisor rollback | unchanged | unchanged |
| Underconstrained ambiguity | old `-` to `+` accepted | withdrawn with native witness |

Eight positives cover both directions for i32, u16 and f64, plus i64 production
modules and a Unicode identifier before the edited span. Assertions cover multiple
inputs, zero, negatives, fractional values, integer truncation and u16 boundaries.
Every positive checks the complete source/test/manifest tree against exactly the
one intended token change BEFORE independent verification cargo creates a lockfile.
Six negatives cover macro-only arithmetic, assignment-only arithmetic, constant
zero divisor, two required edits, a missing Div implementation, and inline,
out-of-line and integration test-only arithmetic. No negative edit or rollback leak.

The supplemental variable divisor candidate passes its nonzero test but panics
at zero; refusal leaves every original byte intact. Its failing-test observation
is retained in candidate-supplement/checks. The custom Only type produces E0369
for the first `/` candidate; native search then repairs the later scalar expression.
The compile failure and successful final tests are both retained.

The cross-file fixture has a new `/` proposal in early.rs and an old `-` proposal
in late.rs, both capable of passing the visible test. Baseline and candidate both
select the exact old late.rs patch. A library test additionally checks old ordering
across lines/files and exact Unicode character spans, with nonexpression exclusions.

Ambiguity fixture: original `x / 2 - 3`, visible input 4 expects 5. Old `x / 2 + 3`
and new `x * 2 - 3` both pass. The unchanged guard executes native probes at -128,
confirming -61 versus -259 and withdrawing the patch. Original code/tests remain
unchanged. This is an explicitly justified correctness withdrawal, not a regression
hidden to improve a count. `reports/native-ambiguity.log` retains the full witness.

Validation: all 39 world-rust tests passed (34 library, 4 isolation, 1 new proposal
test); scoped clippy all-targets with warnings denied, fmt check, release main build,
and git diff --check passed. No tests were weakened. Original runtime tests retained.

## TRAIN evidence and limitations

The fixed public TRAIN sample is four available rb_fix_test crates. Full affected
TRAIN is all 27 available crates selected by the unchanged coding-repair-dev adapter.
Only label kind/split and turn identifiers were used, plus input source/request fields.
Public source/tests inspected included download-progress (reversed arguments),
thermostat-fahrenheit (integer grouping), and shipping-boxes (ceiling division).
These are not correct single multiply/divide repairs. No expected patches, official
or held-out tests, hidden generators, private historical logs or parent evaluator read.

The sample remains 2 verified / 2 refused. All27 remain 5 verified / 22 refused,
with zero changed verdicts OR proposed patches. See reports/summary.json and the four
TRAIN reports. This measures visible native cargo verification only. Zero TRAIN gain.
The supplied parent baseline 200/765 and fix-test15/81 was not re-evaluated here.
The parent must check all765 unchanged passes/failures before accepting release.

No general semantic correctness or absence of untested division-by-zero/overflow
is proved. Float and complex-expression ambiguity remain outside the existing
witness subset. Both-direction coverage is authored development evidence, not an
independent benchmark. Old candidates can exhaust the unchanged search budget;
new candidates intentionally get no priority over them.

## Versions, hashes and provenance

Original binary SHA256:
`30c25729e302578be2091071572201b389ebe72df98ba059ed748b0fe491dbb1`.
Candidate binary SHA256:
`deec095b2261610795dd4ebbe71851d378b698710d5135256a574f4668192f06`.
Production-only patch SHA256:
`ae506e510cb541157bb38e967b2cd702f6e00e1998cabfdf51fd0fdc6f0feb16`.

Candidate source SHA256:
- checks.rs: `44c834a1f4ebd5e681957e4885b5430a4125cccaebe050a24f4fedb1f398ff07`
- slips.rs: `74cc8d64bd4708f0b504622bf26b710055f7c1f0e45809357d9d75549f776a50`
- ambiguity.rs (UNCHANGED): `bb0e2d7aafcaf5bd04004c28003289760f5cec927ff3e9042bd9a12938108795`
- tests/multiplicative.rs: `b5de1e6b9a1b20368f7fd36081984789d62807b598a465630394c05636223c9b`

No external downloads, new dependencies, runtime models or lookup tables. Authored
fixtures use repository MIT OR Apache-2.0 license; version is the two frozen plans
on 2026-10-11. fixtures.json = 8,692 bytes, supplement.json = 1,095 bytes; embedded
source/manifest/test payload totals 5,157 bytes. Per-file hashes in PLAN.sha256,
SUPPLEMENT.sha256 and reports/fixture-provenance.json. Supplied TRAIN snapshot
`train-v3`: 108 selected public Rust/manifest/lock files, 29,232 bytes, exact hashes
in reports/train-source-manifest.json; every original hash rechecked unchanged.
No external licensing inference or redistribution claim for the supplied TRAIN data.

## Reproduction and retained failures

Run from this checkout, supplying the existing target and persistent disk TMPDIR:

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/multiplicative-run/target
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/multiplicative-run/tmp
export BASELINE=/home/gsc/.cache/glassmind/continuous-20261010/multiplicative-run/baseline/glassmind
export OUT=/home/gsc/.cache/glassmind/continuous-20261010/multiplicative-run/replay-new
bash tools/coding-multiplicative/reproduce.sh
python3 tools/coding-multiplicative/summarize.py "$OUT"
```

The script documents commands run individually, not a claimed second full replay.
Every cargo invocation, including native subprocesses, uses persistent TMPDIR,
systemd user scope MemoryMax=8G/MemorySwapMax=0, setsid --wait, timeout
--kill-after=10s 600, -j1 and --test-threads 4 when testing. Existing native wrapper
also retains its tighter ambiguity deadlines. Offline candidate/build validation.

Evidence root:
`/home/gsc/.cache/glassmind/continuous-20261010/multiplicative-run/evidence`.
Includes exact authored inputs, CLI stdout/stderr, before/after file hashes,
independent native verification logs, cached failed candidate diagnostics, witness,
TRAIN inputs and canonical outcome JSON, stderr logs, checks, source manifests.
TRAIN stdout is persisted by the existing adapter as parsed/canonical JSON rather
than original stdout whitespace. Authored CLI stdout/stderr is retained verbatim.
No missing binary baseline was substituted for an actual run.

Failed experiment/evidence: no runtime variant was rejected or retuned. The first
build command used nonexistent package name glassmind-cli and failed before compiling;
reports/build.log retains that error, corrected to package glassmind in
reports/build-corrected.log. Expected native failure cases are retained, not erased.
No agents, additional LLM calls, services, credentials/API keys or commits used.
No owned jobs remain after handoff.

Original submission is preserved under sibling run/frozen-candidate: source files,
this report, complete submission.patch, production runtime.patch, exact candidate
binary and all evidence, with SHA256 manifest. Parent can review/integrate or reject
without losing the submitted candidate. Next action: parent full765 regression
check, then decide whether measured authored capability with zero TRAIN gain merits
integration. No acceptance on benchmark correctness is claimed by this worker.
