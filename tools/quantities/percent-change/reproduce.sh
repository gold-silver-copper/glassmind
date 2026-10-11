#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
RUN=/home/gsc/.cache/glassmind/continuous-20261010/percentchange-run
export TMPDIR="$RUN/tmp" GLASSMIND_SCRATCH="$RUN/scratch" CARGO_TARGET_DIR="$RUN/target"
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
OUT="${1:-$RUN/reproduction}"
mkdir -p "$OUT"
# No new downloads. Validation wraps EVERY cargo call with the required scope.
bash tools/quantities/percent-change/validate.sh > "$OUT/validation.log" 2>&1
cp "$RUN/baseline/quantities" "$TMPDIR/glassmind-quantities-baseline"
python3 tools/quantities/percent-change/probes.py "$RUN/baseline/quantities" "$CARGO_TARGET_DIR/debug/glassmind-quantities" "$OUT/probes.json"
python3 tools/quantities/linearwords/check.py > "$OUT/regressions.log"
cp "$TMPDIR/checks.json" "$OUT/regressions.json"
python3 tools/quantities/linearwords/absolute.py --binary "$CARGO_TARGET_DIR/debug/glassmind-quantities" --output "$OUT/absolute.json"
for sample in sample full; do
 args=(); if [[ "$sample" == sample ]]; then args+=(--sample); fi
 python3 tools/quantities/percent-change/evaluate.py --binary "$CARGO_TARGET_DIR/debug/glassmind-quantities" --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa --output "$OUT/$sample" "${args[@]}"
done
