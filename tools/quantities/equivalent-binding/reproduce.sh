#!/usr/bin/env bash
set -euo pipefail
: "${EQUIV_RUN:?Choose a fresh persistent experiment directory}"
: "${CARGO_TARGET_DIR:?Reuse the existing target directory}"
: "${TMPDIR:?Use persistent storage outside /tmp}"
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
mkdir -p "$EQUIV_RUN"
cp tools/quantities/equivalent-binding/PLAN.md "$EQUIV_RUN/PLAN.md"
python tools/quantities/equivalent-binding/prepare.py > "$EQUIV_RUN/prepare.log"
python tools/quantities/equivalent-binding/build.py
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test --offline --release --manifest-path "$EQUIV_RUN/helper/Cargo.toml" -j1 -- --test-threads 4 > "$EQUIV_RUN/checks.log" 2>&1
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build --offline --release --manifest-path "$EQUIV_RUN/helper/Cargo.toml" -j1 > "$EQUIV_RUN/build.log" 2>&1
python tools/quantities/equivalent-binding/run.py > "$EQUIV_RUN/pipeline.log" 2>&1
python tools/quantities/equivalent-binding/verify.py > "$EQUIV_RUN/verify.log" 2>&1
