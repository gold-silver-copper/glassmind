#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
REVIEW=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/indexproof-review
OWNRUN=/home/gsc/.cache/glassmind/continuous-20261010/indexproof-run
INDEPENDENT=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/indexproof-independent
export TMPDIR="$REVIEW/tmp" GLASSMIND_SCRATCH="$REVIEW/scratch"
export CARGO_TARGET_DIR="$REVIEW/native-target" GLASSMIND_CARGO_TARGET="$REVIEW/native-target"
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=4
unset GLASSMIND_CHECK_CACHE
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap python3 -B tools/coding-index-proof/run.py --binary "$REVIEW/glassmind-integrated" --dev-inputs "$OWNRUN/dev-inputs" --out "$REVIEW/sample" --sample > "$REVIEW/sample.log" 2>&1
cap python3 -B tools/coding-index-proof/run.py --binary "$REVIEW/glassmind-integrated" --dev-inputs "$OWNRUN/dev-inputs" --out "$REVIEW/full" > "$REVIEW/full.log" 2>&1
cap python3 -B "$INDEPENDENT/run.py" --binary "$REVIEW/glassmind-integrated" --out "$INDEPENDENT/integrated" > "$REVIEW/independent.log" 2>&1
cap python3 -B "$INDEPENDENT/cross-target.py" --binary "$REVIEW/glassmind-integrated" --out "$INDEPENDENT/wasm-integrated" > "$REVIEW/wasm.log" 2>&1
echo 'owned32, independent11 and actual32bit-target4 complete'
