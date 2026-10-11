#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${CARGO_TARGET_DIR:?Use the coordinator-provided existing cargo target directory}"
export TMPDIR="${QA_TMPDIR:-$PWD/../readingqa-run/tmp/qa}"
export QA_ARTIFACTS_DIR="${QA_ARTIFACTS_DIR:-$TMPDIR/artifacts}"
export QA_REPORT_DIR="${QA_REPORT_DIR:-$TMPDIR/reports}"
mkdir -p "$TMPDIR"
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-reading -p glassmind -- --check
cap cargo test -j1 -p glassmind-reading --all-targets -- --test-threads 4
cap cargo clippy -j1 -p glassmind-reading -p glassmind --all-targets -- -D warnings
cap cargo build --release -j1 -p glassmind-reading -p glassmind --bins
binary="$CARGO_TARGET_DIR/release/reading-qa"
cap python tools/reading/qa/run.py prepare
cap "$binary" train --input "$QA_ARTIFACTS_DIR/fit.train.jsonl" --model "$QA_ARTIFACTS_DIR/model.json"
cap python tools/reading/qa/run.py calibrate --binary "$binary"
cap python tools/reading/qa/run.py sample --binary "$binary"
cap python tools/reading/qa/run.py full --binary "$binary"
cap python tools/reading/qa/run.py independent --binary "$binary"
cap "$CARGO_TARGET_DIR/release/glassmind" read --model "$QA_ARTIFACTS_DIR/model.json" --input "$QA_ARTIFACTS_DIR/full.input.jsonl" > "$QA_ARTIFACTS_DIR/main.full.outputs.jsonl"
cmp "$QA_ARTIFACTS_DIR/main.full.outputs.jsonl" "$QA_ARTIFACTS_DIR/full.candidate.outputs.jsonl"
contracts=$(mktemp -d "$TMPDIR/qa-contracts.XXXXXX")
cap python tools/reading/qa/check-main.py --binary "$CARGO_TARGET_DIR/release/glassmind" --model "$QA_ARTIFACTS_DIR/model.json" --output "$contracts/results"
cap python tools/reading/qa/attest.py
