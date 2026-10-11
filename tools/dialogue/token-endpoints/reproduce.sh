#!/usr/bin/env bash
set -euo pipefail
ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
RUN=${TOKEN_ENDPOINT_RUN:-/home/gsc/.cache/glassmind/continuous-20261010/tokenendpoints-run}
SGD_ROOT=${SGD_ROOT:-/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full}
export TMPDIR="$RUN/tmp"
export GLASSMIND_SCRATCH="$RUN/scratch"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$RUN/target}
OWN=tools/dialogue/token-endpoints
OUT="$RUN/reproduction"
mkdir -p "$TMPDIR" "$GLASSMIND_SCRATCH" "$OUT"
BIN_SOURCE=crates/dialogue/src/bin/dialogue-token-endpoint-trace.rs
test ! -e "$BIN_SOURCE"
cp "$OWN/diagnostic.rs" "$BIN_SOURCE"
trap 'rm -f "$BIN_SOURCE"' EXIT
cap() {
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
python3 -B "$OWN/prepare.py" --root "$SGD_ROOT" --output "$OUT/train-input.json"
python3 -B -m unittest discover -s "$OWN" -p test_experiment.py -v
cap python3 -B "$OWN/experiment.py" discover --root "$SGD_ROOT" --output "$OUT"
cap cargo build --offline --release -p glassmind-dialogue --bin dialogue-token-endpoint-trace -j1
cap "$CARGO_TARGET_DIR/release/dialogue-token-endpoint-trace" \
  "$RUN/baseline/model.json" "$SGD_ROOT/selected/train-schema.json" \
  "$OUT/train-input.json" "$OUT/train-reference-proposals.jsonl"
cap python3 -B "$OWN/experiment.py" audit --root "$SGD_ROOT" --output "$OUT" \
  --trace "$OUT/train-reference-proposals.jsonl"
cmp "$OWN/reports/candidates.json" "$OUT/candidates.json"
cmp "$OWN/reports/gate.json" "$OUT/gate.json"
cmp "$OWN/reports/audit.jsonl" "$OUT/audit.jsonl"
cmp "$OWN/reports/train-reference-proposals.jsonl" "$OUT/train-reference-proposals.jsonl"
