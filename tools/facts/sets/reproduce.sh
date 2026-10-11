#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent owned path required}"
: "${CARGO_TARGET_DIR:?existing target directory required}"
: "${BASELINE_DIR:?directory containing frozen baseline glassmind and continuous_predictions}"
FACTS_DATA=${FACTS_DATA:-/home/gsc/.cache/glassmind/data/simplequestions}
RUN=${RUN:-$TMPDIR/sets-replay}
mkdir -p "$RUN"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap python3 tools/facts/chat/package.py --index tools/facts/index.json --model "$FACTS_DATA/relation.json" --source 'Wikidata frozen 2026-10-10' --license 'CC0-1.0; model from SimpleQuestions CC-BY-3.0' --scope '500 validation-subject-seeded entities; finite demonstration' --output "$RUN/bundle"
cap python3 tools/facts/sets/contracts.py --binary "$BASELINE_DIR/glassmind" --baseline --output "$RUN/baseline-contracts"
cap python3 tools/facts/sets/regression.py --binary "$BASELINE_DIR/glassmind" --predictor "$BASELINE_DIR/continuous_predictions" --bundle "$RUN/bundle" --valid "$FACTS_DATA/valid.tsv" --output "$RUN/baseline"
cap cargo fmt -p glassmind -p glassmind-facts --check
cap cargo test --release -p glassmind -p glassmind-facts -j1 -- --test-threads 4
cap cargo clippy -p glassmind -p glassmind-facts --all-targets -j1 -- -D warnings
cap cargo build --release -p glassmind -p glassmind-facts --bins --example continuous_predictions -j1
cap python3 tools/facts/sets/contracts.py --binary "$CARGO_TARGET_DIR/release/glassmind" --output "$RUN/contracts"
cap python3 tools/facts/sets/session_contracts.py --binary "$CARGO_TARGET_DIR/release/glassmind" --bundle "$RUN/contracts/bundle" --output "$RUN/session-contracts"
cap python3 tools/facts/sets/regression.py --binary "$CARGO_TARGET_DIR/release/glassmind" --predictor "$CARGO_TARGET_DIR/release/examples/continuous_predictions" --bundle "$RUN/bundle" --valid "$FACTS_DATA/valid.tsv" --output "$RUN/candidate" --baseline "$RUN/baseline"
cap python3 tools/facts/chat/legacy.py --binary "$CARGO_TARGET_DIR/release/glassmind" --facts-dir "$RUN/bundle" --output "$RUN/legacy"
cap python3 tools/facts/chat/legacy.py --binary "$CARGO_TARGET_DIR/release/glassmind" --output "$RUN/legacy-default"
cap python3 tools/facts/sets/real_example.py --binary "$CARGO_TARGET_DIR/release/glassmind" --bundle "$RUN/bundle" --cache "$FACTS_DATA/wikidata" --output "$RUN/real-example"
