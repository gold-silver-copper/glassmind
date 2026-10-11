#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?set persistent non-/tmp directory}" "${CARGO_TARGET_DIR:?use existing target}"
MATCH_ROOT=tools/facts/matching
"$MATCH_ROOT/capped.sh" cargo test --offline -p glassmind-facts -j1 -- --test-threads 4
"$MATCH_ROOT/capped.sh" cargo clippy --offline -p glassmind-facts --all-targets -j1 -- -D warnings
"$MATCH_ROOT/capped.sh" cargo build --offline --release -p glassmind-facts --bin glassmind-facts --example continuous_predictions -j1
"$MATCH_ROOT/final_gates.sh"
"$MATCH_ROOT/capped.sh" python "$MATCH_ROOT/legacy_queries.py" --baseline "$MATCH_ROOT/run/baseline/glassmind-facts" --candidate "$CARGO_TARGET_DIR/release/glassmind-facts" --output "$MATCH_ROOT/run/legacy-queries-replay"
"$MATCH_ROOT/capped.sh" python "$MATCH_ROOT/legacy_sets.py"
"$MATCH_ROOT/capped.sh" python "$MATCH_ROOT/native_invariants.py" --binary "$CARGO_TARGET_DIR/release/glassmind-facts" --output "$MATCH_ROOT/run/native-invariants-replay"
