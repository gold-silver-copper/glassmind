#!/usr/bin/env bash
set -uo pipefail
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/editlanguage-check
export CARGO_TARGET_DIR="$TMPDIR/cargo-target" GLASSMIND_SCRATCH="$TMPDIR/scratch"
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=4
b=/home/gsc/code/glassmind/target/release/glassmind
i=/home/gsc/.cache/glassmind/continuous-20261010/editlanguage-run/dev-inputs
python3 tools/coding-addfields-dev.py --binary "$b" --dev-inputs "$i" --output "$TMPDIR/sample" --sample > "$TMPDIR/sample.log" 2>&1
printf 'sample=%s\n' "$?"
python3 tools/coding-addfields-dev.py --binary "$b" --dev-inputs "$i" --output "$TMPDIR/full" > "$TMPDIR/full.log" 2>&1
printf 'full=%s\n' "$?"
python3 tools/coding-editlanguage/probes.py --binary "$b" --output "$TMPDIR/independent" > "$TMPDIR/independent.log" 2>&1
printf 'independent=%s\n' "$?"
python3 tools/coding-metatypes-dev.py --binary "$b" --output "$TMPDIR/metatypes" > "$TMPDIR/metatypes.log" 2>&1
printf 'metatypes=%s\n' "$?"
python3 tools/coding-answerfiles-regression.py --binary "$b" --output "$TMPDIR/queries" > "$TMPDIR/queries.log" 2>&1
printf 'queries=%s\n' "$?"
