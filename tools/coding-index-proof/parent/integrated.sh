#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
REVIEW=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/indexproof-review
export TMPDIR="$REVIEW/tmp"
export GLASSMIND_SCRATCH="$REVIEW/scratch"
export CARGO_TARGET_DIR=/home/gsc/code/glassmind/target
export GLASSMIND_CARGO_TARGET="$REVIEW/native-target"
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=4
mkdir -p "$TMPDIR" "$GLASSMIND_SCRATCH"
unset GLASSMIND_CHECK_CACHE
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind-world-rust -j1 -- --test-threads 4 > "$REVIEW/world-tests.log" 2>&1
bash tools/check.sh > "$REVIEW/workspace.log" 2>&1
cp target/release/glassmind "$REVIEW/glassmind-integrated"
sha256sum "$REVIEW/glassmind-integrated" > "$REVIEW/binary.sha256"
echo 'release and workspace checks passed'
