#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent owned path required}"
: "${CARGO_TARGET_DIR:?existing target required}"
: "${BASELINE_DIR:?directory containing retained standalone-2821.jsonl}"
FACTS_DATA=${FACTS_DATA:-/home/gsc/.cache/glassmind/data/simplequestions}
RUN=${RUN:-$TMPDIR/verification-replay}
mkdir -p "$RUN"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
sha256sum -c tools/facts/verification/frozen.sha256 tools/facts/verification/supplemental.sha256
cap cargo fmt -p glassmind-facts --check
cap cargo clippy -p glassmind-facts --all-targets -j1 -- -D warnings
cap cargo build --release -p glassmind-facts --bins --example continuous_predictions -j1
python3 tools/facts/verification/archive.py --target "$CARGO_TARGET_DIR" --output "$RUN/frozen-candidate"
cap cargo test --release -p glassmind-facts -j1 -- --test-threads 4
cap python3 tools/facts/verification/contracts.py --binary "$RUN/frozen-candidate/glassmind-facts" --output "$RUN/contracts"
cap python3 tools/facts/verification/legacy.py --binary "$RUN/frozen-candidate/glassmind-facts" --predictor "$RUN/frozen-candidate/continuous_predictions" --baseline "$BASELINE_DIR/standalone-2821.jsonl" --baseline-binary "$BASELINE_DIR/glassmind-facts" --data "$FACTS_DATA" --index tools/facts/index.json --output "$RUN/legacy"
cap python3 tools/facts/verification/real_example.py --binary "$RUN/frozen-candidate/glassmind-facts" --index tools/facts/index.json --cache "$FACTS_DATA/wikidata" --output "$RUN/real-example"
