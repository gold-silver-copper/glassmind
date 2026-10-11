# Typed quantity comparison handoff

Completed one bounded batch; uncommitted, based on cddb06558a3c580392a7b678187cbb35cb2b6709. Only crates/quantities and tools/quantities/unit-comparison changed. Parent owns conversation integration. No agents, training, model calls, downloads, services, external/session state writes, or hidden/official-test material. No broad word-problem accuracy claim.

## Changes and API

- New `glassmind_quantities::comparison`: `ComparisonQuantity::new(Rational, unit_name)`, public `ComparisonQuery { left, right, operation }`, `query.evaluate()`, `parse_comparison`, and `compare_question`. Predicate, selection, ordering, selected operand, result and refusal are Rust types. Quantity fields are private; evaluation checks matching dimensions and selection adjective even for direct queries.
- Result includes exact operands, dimension, ordering, predicate/truth or selection/left/right/equal, and conversion witness (left operand's unit and both exact magnitudes). Rational JSON fields are strings. Equal selections explicitly return `equal`, meaning both. No boolean is encoded as scalar `Answer`.
- `Rational::checked_cmp` and `Quantity<D>::checked_cmp` use checked exact arithmetic. Existing Unit scale/offset definitions and `Unit::convert` are unchanged. Both operands pass through existing conversion; no raw-numeral shortcut or duplicated unit scales.
- Fully consumed grammar in frozen PLAN.md. 4096 UTF-8 byte input limit, NUL rejection, 100-byte number-token bound, fixed linear scans, exact signed integer/decimal/fraction literals, case/whitespace normalization, multiword unit names, speed slash units. Optional single final question mark; comma-adjacent whitespace can be omitted. No implicit unit carry, unitless arithmetic, compound expressions, number words, pronouns or extra assertions.
- Independent native `--compare-jsonl` streams JSON strings to `{status: comparison, result: ...}` or `{status: refusal, error: {kind: ...}}`. Bounded line storage (24580 bytes including transport punctuation/CRLF), malformed/oversize lines refuse and next line resumes. Existing `--jsonl`, scalar `solve`, `Answer`, and old grammar unchanged.
- Public native `examples/compare_typed.rs` accepts JSON `{left:{value,unit}, right:{value,unit}, operation:{kind:"predicate"|"selection",operation:...}}` and calls the direct API, independent of English parsing. This review adapter expects well-formed structured JSON; it is not the bounded question transport.

Example: `Is 2 kg greater than 1500 g?` returns mass/greater/greater_than/true, operands 2 kg and 1500 g, witness 2 and 3/2 kg. `Which is longer, 3 feet or 1 metre?` selects right with witness 3 and 1250/381 feet. `Is 1/3 hour equal to 20 minutes?` returns equality/true and 1/3,1/3 hour.

## Frozen baseline and validation

PLAN.md, freeze.py and authored.json were frozen and SHA256-recorded before production edits. The supplied original native solver refused all 64 authored questions. Candidate: 32 correct substantive comparisons, 32 expected refusals, zero wrong; all 32 substantive results exactly match independent direct-API adapter calls. The public authored expectations include false/equality/dimension/affine/negative/fraction/case-space and unsupported controls. Python Fraction independently checks witness ordering.

Final native package tests: 19 passed (12 existing + 7 new test functions). New tests cover every predicate in all six dimensions at less/equal/greater, inverse facts, all eight adjectives including equality, typed dimension validation, statically typed affine equality, exact-rational ordering, checked overflow/zero denominator, full-consumption/input boundaries and streaming recovery. Additional five native transport cases pass.

`fmt --check`, package `clippy --all-targets -- -D warnings`, and package release all-target build pass. All cargo calls used persistent TMPDIR, the existing CARGO_TARGET_DIR, systemd user scopes with MemoryMax=8G/MemorySwapMax=0, setsid --wait, timeout --kill-after=10s 600, -j1 (except fmt), --test-threads 4 for tests. Outer scope was verified capped at 8589934592 bytes and zero swap. No jobs remain running.

Ordinary solver compatibility ran first200 BEFORE full4475, then authoredsynthetic3116. All bytes match supplied baseline, no answer labels used:

| Input | Rows | Output bytes | Output SHA256 |
|---|---:|---:|---|
| Public MathQA DEV first200 | 200 | 11474 | dc6b9f8d10bcf64c23295e666e67edd63f10616ad9068b9b34458a87aa0a85bf |
| Public MathQA DEV full | 4475 | 258256 | 5cf77234ace536a7f147c3a1bc4ac832a1052e2ac50c05ea232a1ce4a175eb98 |
| Existing authored synthetic | 3116 | 296711 | ab818fdee8b4b383c8e0439dae83d9136926c6a5b4558c66033dfff4dac895ab |

Initial failure retained: clippy rejected the initial wire response enum (400-byte successful variant vs 32-byte refusal). Boxed the successful payload; JSON and behavior unchanged. Initial source/log retained under RUN/failures/initial-clippy and copied into frozen-candidate. No failed behavioral hypotheses or rejected learned experiments were introduced; frozen expectations were never changed.

## Data and artifacts

No new external data (0 downloaded bytes). Existing unit definitions: repository base above, MIT OR Apache-2.0. New authored expectations v1: 64 cases, 18872 UTF-8 bytes, authored in this batch under repository MIT OR Apache-2.0; generator freeze.py retained. Existing question-only MathQA DEV revision fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6, Apache-2.0: first200 34213 bytes, full4475 763658 bytes. Existing authored question-only synthetic3116: 163650 bytes, MIT OR Apache-2.0. Original source manifest and exact hashes retained. No labels supplied or needed.

RUN=/home/gsc/.cache/glassmind/continuous-20261010/unitcompare-run. RUN/frozen-candidate contains original and final source, original/final native executables, direct adapter, complete patch including new files, frozen plan/authored expectations, question-only compatibility inputs/baselines, native outputs, reports, commands, all check logs, initial failure, and SHA256/byte manifests. Final native SHA256 a135ff47690658ed9f85e88dc3cea145ac6e2c1b33d624ee8fff9aec71f800e5; direct adapter SHA256 0d5e1cbb059ccc473b5a8d83cefeac2dde00cc13cefcfe6cc44f777bc14b14ed. report.json in this directory is the compact machine-readable result.

## Reproduction

Run from the workspace root with the existing target and persistent temporary directory:

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/unitcompare-run/target
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/unitcompare-run/tmp
tools/quantities/unit-comparison/cargo-checks.sh test
tools/quantities/unit-comparison/cargo-checks.sh fmt
tools/quantities/unit-comparison/cargo-checks.sh clippy
tools/quantities/unit-comparison/cargo-checks.sh release
python3 tools/quantities/unit-comparison/review.py \
  --binary "$CARGO_TARGET_DIR/release/glassmind-quantities" \
  --typed "$CARGO_TARGET_DIR/release/examples/compare_typed" \
  --source /home/gsc/.cache/glassmind/continuous-20261010/unitcompare-source \
  --out /home/gsc/.cache/glassmind/continuous-20261010/unitcompare-run/reproduction
```

The wrapper sets TMPDIR on every cargo invocation and enforces all requested resource limits. To replay without compiling, substitute frozen-candidate/final-native and frozen-candidate/compare-typed for the two binary paths, and frozen-candidate/compatibility-inputs for source.

## Limits and next integration action

Only the documented literal grammar and existing unit names are supported. Signed values are mathematical magnitudes; there is no extra nonnegative/absolute-zero policy. Checked i128 arithmetic may explicitly refuse representable final results when conversion/comparison intermediates overflow. Units/operands are normalized, not preserved as original text spans. Parse produces a typed query; evaluate performs dimension validation. No main conversation integration was attempted. Parent can call comparison::compare_question and render the typed outcome/witness without converting it to scalar Answer. Compatibility evidence is unchanged ordinary DEV output, not a new MathQA accuracy measurement. Work is complete within this worker's scope and left uncommitted.
