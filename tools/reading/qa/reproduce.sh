#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${CARGO_TARGET_DIR:?Use the coordinator-provided existing cargo target directory}"
export TMPDIR="${QA_TMPDIR:-$PWD/../readingqa-run/tmp/qa}"
mkdir -p "$TMPDIR"
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-reading -- --check
cap cargo test -j1 -p glassmind-reading --all-targets -- --test-threads 4
cap cargo clippy -j1 -p glassmind-reading --all-targets -- -D warnings
cap cargo build --release -j1 -p glassmind-reading --bin reading-qa
binary="$CARGO_TARGET_DIR/release/reading-qa"
cap python tools/reading/qa/run.py prepare
cap "$binary" train --input tools/reading/qa/artifacts/fit.train.jsonl --model tools/reading/qa/artifacts/model.json
cap python tools/reading/qa/run.py calibrate --binary "$binary"
cap python tools/reading/qa/run.py sample --binary "$binary"
cap python tools/reading/qa/run.py full --binary "$binary"
cap python tools/reading/qa/run.py independent --binary "$binary"
cap python tools/reading/qa/attest.py
