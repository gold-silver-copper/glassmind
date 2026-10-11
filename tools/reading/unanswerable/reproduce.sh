#!/usr/bin/env bash
set -euo pipefail
: "${CARGO_TARGET_DIR:?reuse the provided target directory}"
: "${TMPDIR:?must be an on-disk directory outside /tmp}"
export TMPDIR
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-reading -- --check
cap cargo build -p glassmind-reading --bin reading-qa --release -j1
cap cargo test -p glassmind-reading --lib qa:: -j1 -- --test-threads 4
for stage in prepare train calibrate probes; do cap python3 tools/reading/unanswerable/experiment.py "$stage"; done
if python3 -c 'import json; assert json.load(open("tools/reading/unanswerable/reports/calibration.json"))["gate_pass"]'; then
 cap python3 tools/reading/unanswerable/experiment.py sample
 cap python3 tools/reading/unanswerable/experiment.py full
fi
