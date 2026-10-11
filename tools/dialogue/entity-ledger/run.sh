#!/usr/bin/env bash
set -euo pipefail
export TMPDIR="${TMPDIR:?persistent TMPDIR required}"
: "${CARGO_TARGET_DIR:?existing target required}"
mkdir -p "$TMPDIR"
printf '%q ' "$@" >> "${ENTITY_LEDGER_COMMAND_LOG:-tools/dialogue/entity-ledger/reports/commands.log}"
printf '\n' >> "${ENTITY_LEDGER_COMMAND_LOG:-tools/dialogue/entity-ledger/reports/commands.log}"
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"
