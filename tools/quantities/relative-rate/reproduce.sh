#!/usr/bin/env bash
# Retained rejected candidate replay; never applies the runtime patch.
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${TMPDIR:?Set owned on-disk TMPDIR}" "${CARGO_TARGET_DIR:?Keep existing target}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
export TMPDIR CARGO_TARGET_DIR
WORK=/home/gsc/.cache/glassmind/continuous-20261010
export PYTHONPATH="$WORK/quantities-run/tmp/quantities-deps"
BINARY="$WORK/relativerate-run/candidate/glassmind-quantities"
DATA="$WORK/quantities-run/tmp/mathqa"
OUT="${RELATIVE_REPLAY_OUT:-$WORK/relativerate-run/replay}"
mkdir -p "$OUT"
python3 tools/quantities/relative-rate/evaluate.py --binary "$BINARY" --data "$DATA" --output "$OUT/sample" --sample
python3 tools/quantities/relative-rate/evaluate.py --binary "$BINARY" --data "$DATA" --output "$OUT/full"
python3 tools/quantities/relative-rate/probes.py "$WORK/relativerate-run/baseline/ratewords" "$BINARY" "$OUT/probes.json"
python3 tools/quantities/ratewords/probes.py --binary "$BINARY" --output "$OUT/old-ratewords-probes.json"
cp "$WORK/relativerate-run/baseline/production" "$TMPDIR/glassmind-quantities-baseline"
CANDIDATE_BINARY="$BINARY" python3 tools/quantities/linearwords/check.py
cp "$TMPDIR/checks.json" "$OUT/regressions.json"
python3 tools/quantities/linearwords/absolute.py --binary "$BINARY" --output "$OUT/absolute.json"
