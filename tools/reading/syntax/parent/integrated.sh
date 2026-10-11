#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
REVIEW=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/udsyntax-review
export TMPDIR="$REVIEW/tmp" GLASSMIND_SCRATCH="$REVIEW/scratch"
export CARGO_TARGET_DIR=/home/gsc/code/glassmind/target
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=4
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind-reading -j1 -- --test-threads 4 > "$REVIEW/reading-tests.log" 2>&1
bash tools/check.sh > "$REVIEW/workspace.log" 2>&1
cp target/release/glassmind "$REVIEW/glassmind-integrated"
cp target/release/reading-syntax "$REVIEW/reading-syntax-integrated"
sha256sum "$REVIEW/glassmind-integrated" "$REVIEW/reading-syntax-integrated" > "$REVIEW/binaries.sha256"
echo 'reading tests and workspace checks passed'
