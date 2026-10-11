#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent TMPDIR required}" "${CARGO_TARGET_DIR:?existing target required}"
exec systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo "$@"
