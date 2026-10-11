#!/usr/bin/env bash
# Fresh output directory required; official TRAIN only, unchanged original executor.
set -euo pipefail
SOURCE=${1:?usage: reproduce.sh SOURCE NEW_OUTPUT}
AUDIT_OUTPUT=${2:?usage: reproduce.sh SOURCE NEW_OUTPUT}
ADAPTER_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
mkdir -p "$AUDIT_OUTPUT/tmp"
export TMPDIR="$AUDIT_OUTPUT/tmp"
export PYTHONDONTWRITEBYTECODE=1
for mode in contracts sample full; do
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 \
    python3 "$ADAPTER_DIR/audit.py" --source "$SOURCE" \
    --output "$AUDIT_OUTPUT/$mode" --mode "$mode"
done
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  python3 "$ADAPTER_DIR/verify_evidence.py" --source "$SOURCE" \
  --frozen "$AUDIT_OUTPUT" --output "$AUDIT_OUTPUT/evidence-verification.json"
