#!/usr/bin/env bash
set -euo pipefail
BINARY=$1
VERSION=$2
OUT=tools/conversation/social-routing/reports
MODEL=/home/gsc/.cache/glassmind/models/everyday/intent.json
for MODE in plain model; do
 ARGS=()
 if [[ "$MODE" == model ]]; then ARGS=(--model "$MODEL"); fi
 python3 tools/conversation/memory-authority/evaluate.py "$BINARY" "$OUT/$VERSION-memory-$MODE" "${ARGS[@]}"
 python3 tools/conversation/memory-authority/learned_social.py "$BINARY" "$OUT/$VERSION-supplement-$MODE" "${ARGS[@]}"
done
python3 tools/talk-dev.py "$BINARY" --model "$MODEL" --output "$OUT/$VERSION-calc27.json"
python3 tools/talk-story-dev.py "$BINARY" --model "$MODEL" --output "$OUT/$VERSION-story47.json"
python3 tools/facts/sets/contracts.py --binary "$BINARY" --output "$OUT/$VERSION-facts"
python3 tools/facts/sets/session_contracts.py --binary "$BINARY" --bundle "$OUT/$VERSION-facts/bundle" --output "$OUT/$VERSION-facts-session"
