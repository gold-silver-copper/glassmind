#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent TMPDIR required}" "${CARGO_TARGET_DIR:?existing target required}" "${SEQUENCE_RUN:?fresh output directory required}"
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
scope=(systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600)
"${scope[@]}" cargo test -p glassmind-quantities --test sequence -j1 -- --test-threads 4
"${scope[@]}" cargo build --release -p glassmind-quantities --examples --bin glassmind-quantities -j1
"${scope[@]}" python tools/quantities/equation-sequence/experiment.py
