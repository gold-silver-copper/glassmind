#!/usr/bin/env bash
# Replay evidence using the retained, frozen rejected binary; never applies patch.
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${TMPDIR:?Set owned disk temporary directory}"
: "${CARGO_TARGET_DIR:?Keep existing target directory}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
export TMPDIR CARGO_TARGET_DIR
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
RUN=/home/gsc/.cache/glassmind/continuous-20261010/ratewords-run
DATA=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa
export CANDIDATE_BINARY="${CANDIDATE_BINARY:-$RUN/candidate/glassmind-quantities}"
OUT="${RATEWORDS_REPLAY_OUT:-$RUN/replay}"
mkdir -p "$OUT"
cp "$RUN/baseline/glassmind-quantities" "$TMPDIR/glassmind-quantities-baseline"
capped() {
    TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=10s 600 "$@"
}
capped python3 tools/quantities/evaluate_mathqa.py --data "$DATA" --binary "$CANDIDATE_BINARY" --sample --report "$OUT/sample.json"
capped python3 tools/quantities/evaluate_mathqa.py --data "$DATA" --binary "$CANDIDATE_BINARY" --report "$OUT/full.json"
capped python3 tools/quantities/ratewords/audit.py --baseline "$RUN/baseline/full.json" --candidate "$OUT/full.json" --output "$OUT/audit.json"
capped python3 tools/quantities/ratewords/probes.py --binary "$CANDIDATE_BINARY" --output "$OUT/probes.json"
capped python3 tools/quantities/linearwords/check.py
cp "$TMPDIR/checks.json" "$OUT/regressions.json"
capped python3 tools/quantities/linearwords/absolute.py --binary "$CANDIDATE_BINARY" --output "$OUT/absolute.json"
