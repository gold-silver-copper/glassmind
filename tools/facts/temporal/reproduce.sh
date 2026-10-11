#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
WORK=$(dirname "$ROOT")
RUN=${RUN:-$WORK/temporal-run}
export TMPDIR="$RUN/tmp"
export CARGO_TARGET_DIR="$RUN/target"
mkdir -p "$TMPDIR" "$RUN/reports"
cd "$ROOT"
capped_cargo() {
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
python3 tools/facts/temporal/extract.py --source "$WORK/temporal-source" --index "$WORK/factmatch-source/index.json" --index-manifest "$WORK/factmatch-source/manifest.json" --output "$RUN/temporal.json"
capped_cargo fmt -p glassmind-facts -- --check
capped_cargo test -p glassmind-facts -j1 -- --test-threads 4
capped_cargo clippy -p glassmind-facts --all-targets -j1 -- -D warnings
capped_cargo build -p glassmind-facts --bins --examples -j1
python3 tools/facts/temporal/extractor_contracts.py "$RUN"
python3 tools/facts/temporal/proof.py --run "$RUN" --source "$WORK/temporal-source" --data "$WORK/factmatch-source"
python3 tools/facts/verification/legacy.py --binary "$CARGO_TARGET_DIR/debug/glassmind-facts" --predictor "$CARGO_TARGET_DIR/debug/examples/continuous_predictions" --baseline "$RUN/baseline/predictions.jsonl" --baseline-binary "$RUN/baseline/glassmind-facts" --data "$WORK/factmatch-source" --index "$WORK/factmatch-source/index.json" --output "$RUN/reports/legacy"
python3 tools/facts/temporal/native_legacy.py "$RUN" "$WORK/factmatch-source"
