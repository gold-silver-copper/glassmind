#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
MATCH_ROOT=tools/facts/matching
MATCH_DATA=/home/gsc/.cache/glassmind/continuous-20261010/factmatch-source
MATCH_BIN="$CARGO_TARGET_DIR/release/glassmind-facts"
"$MATCH_ROOT/capped.sh" cargo fmt -p glassmind-facts -- --check > "$MATCH_ROOT/run/reports/fmt-check.log" 2>&1
"$MATCH_ROOT/capped.sh" python "$MATCH_ROOT/contracts.py" --binary "$MATCH_BIN" --output "$MATCH_ROOT/run/contracts-final" > "$MATCH_ROOT/run/reports/contracts-final.log" 2>&1
"$MATCH_ROOT/capped.sh" "$MATCH_BIN" match --index "$MATCH_ROOT/run/separator-index.json" 'Who was born in Harbor and works for Studio and is a citizen of Republic?' > "$MATCH_ROOT/run/reports/separator-final.json" 2> "$MATCH_ROOT/run/reports/separator-final-scope.log"
"$MATCH_ROOT/capped.sh" python "$MATCH_ROOT/real_example.py" --binary "$MATCH_BIN" --index "$MATCH_DATA/index.json" --output "$MATCH_ROOT/run/reports/real-example-final.json" > "$MATCH_ROOT/run/reports/real-example-final.log" 2>&1
"$MATCH_ROOT/capped.sh" python tools/facts/verification/legacy.py --binary "$MATCH_BIN" --predictor "$CARGO_TARGET_DIR/release/examples/continuous_predictions" --baseline "$MATCH_ROOT/run/baseline/standalone-2821.jsonl" --baseline-binary "$MATCH_ROOT/run/baseline/glassmind-facts" --data "$MATCH_DATA" --index "$MATCH_DATA/index.json" --output "$MATCH_ROOT/run/legacy-final" > "$MATCH_ROOT/run/reports/legacy-final.log" 2>&1
