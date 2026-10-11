#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
RUN="$(cd ../workrates-run && pwd)"
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
OUT="${1:-$RUN/reproduction}"
mkdir -p "$OUT"
capped() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
bash tools/quantities/work-rates/validate.sh > "$OUT/validation.log" 2>&1
cp "$RUN/baseline/quantities" "$TMPDIR/glassmind-quantities-baseline"
capped python tools/quantities/work-rates/probes.py "$CARGO_TARGET_DIR/debug/glassmind-quantities" "$OUT/probes.json"
capped python tools/quantities/linearwords/check.py > "$OUT/regressions.log"
cp "$TMPDIR/checks.json" "$OUT/regressions.json"
capped python tools/quantities/linearwords/absolute.py --binary "$CARGO_TARGET_DIR/debug/glassmind-quantities" --output "$OUT/absolute.json"
capped python tools/quantities/percent-change/probes.py "$RUN/baseline/quantities" "$CARGO_TARGET_DIR/debug/glassmind-quantities" "$OUT/percent-change.json"
capped python tools/quantities/work-rates/regressions.py "$CARGO_TARGET_DIR/debug/glassmind-quantities" "$OUT/parent78.json"
for sample in sample full; do
 args=(); if [[ "$sample" == sample ]]; then args+=(--sample); fi
 capped python tools/quantities/work-rates/evaluate.py --binary "$CARGO_TARGET_DIR/debug/glassmind-quantities" --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa --output "$OUT/$sample" "${args[@]}"
done
