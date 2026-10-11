#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent directory required}"
: "${CARGO_TARGET_DIR:?existing target required}"
: "${QA_SYNTAX_RUN:?fresh external artifact directory required}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap python3 tools/reading/qa-syntax/prepare.py
cap cargo fmt -p glassmind-reading -- --check
cap cargo build --offline --release -p glassmind-reading --bin reading-qa-syntax --bin reading-qa -j1
cap cargo clippy --offline -p glassmind-reading --lib --bin reading-qa-syntax --bin reading-qa --no-deps -j1 -- -D warnings
cap cargo test --offline -p glassmind-reading --lib --bin reading-probability -j1 -- --test-threads 4
cap "$CARGO_TARGET_DIR/release/reading-qa-syntax" --syntax "$QA_SYNTAX_RUN/../baseline/syntax.bin" train --input "$QA_SYNTAX_RUN/fit.train.jsonl" --model "$QA_SYNTAX_RUN/model.json"
cap python3 tools/reading/qa-syntax/run.py calibrate
# Must reject before DEV if selected is null.
python3 -c 'import json; assert json.load(open("tools/reading/qa-syntax/reports/calibration.json"))["selected"] is not None'
for stage in sample full probes; do cap python3 tools/reading/qa-syntax/run.py "$stage"; done
cap python3 tools/reading/qa-syntax/contracts.py
cap python3 tools/reading/qa-syntax/audit.py
