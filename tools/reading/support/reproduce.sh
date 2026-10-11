#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
cd "$ROOT"
: "${CARGO_TARGET_DIR:?Use existing capped target directory}"
: "${TMPDIR:?Use on-disk TMPDIR outside /tmp}"
export SQUAD_SOURCE=${SQUAD_SOURCE:-$ROOT/../readingqa-source}
export SUPPORT_RUN=${SUPPORT_RUN:-$ROOT/../qasupport-run/support}
mkdir -p "$SUPPORT_RUN"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
# If the candidate was rejected, restore its experimental adapter temporarily:
# git apply tools/reading/support/rejected.patch
cap python3 tools/reading/support/prepare.py
cap cargo build --release -p glassmind-reading --bin reading-support --bin reading-qa -j1
cap cargo test --release -p glassmind-reading -j1 -- --test-threads 4
cap python3 tools/reading/support/experiment.py train
cap python3 tools/reading/support/experiment.py sample
cap python3 tools/reading/support/experiment.py full
cap python3 tools/reading/support/experiment.py probes
cap python3 tools/reading/support/audit.py
