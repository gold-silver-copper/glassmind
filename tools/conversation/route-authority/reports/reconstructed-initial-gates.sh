#!/usr/bin/env bash
set -euo pipefail
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/scratch
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
B=../routeauthority-run/baseline/glassmind
C=../routeauthority-run/glassmind-candidate
A=../socialrouting-run/submitted/tools/conversation/social-routing
O=tools/conversation/route-authority/reports
M=/home/gsc/.cache/glassmind/models/everyday/intent.json
case "$1" in
plain|model)
 MODE=$1; ARGS=(); if [[ $MODE == model ]]; then ARGS=(--model "$M"); fi
 if [[ $MODE == plain ]]; then python3 "$A/evaluate.py" "$B" "$O/baseline-old-sample-$MODE" --sample "${ARGS[@]}"; fi
 python3 "$A/evaluate.py" "$C" "$O/candidate-old-sample-$MODE" --sample "${ARGS[@]}"
 python3 "$A/evaluate.py" "$C" "$O/candidate-old-full-$MODE" "${ARGS[@]}"
 python3 tools/conversation/memory-authority/evaluate.py "$C" "$O/candidate-memory-$MODE" "${ARGS[@]}"
 python3 tools/conversation/memory-authority/learned_social.py "$C" "$O/candidate-goodbye-$MODE" "${ARGS[@]}"
 ;;
boundaries)
 for MODE in plain model; do
  ARGS=(); SUFFIX=''; if [[ $MODE == model ]]; then ARGS=(--model "$M"); SUFFIX=-model; fi
  python3 tools/conversation/route-authority/boundaries.py "$C" "$O/candidate-boundary-sample$SUFFIX" --sample --baseline "$O/baseline-boundary-sample$SUFFIX" "${ARGS[@]}"
  python3 tools/conversation/route-authority/boundaries.py "$C" "$O/candidate-boundary$SUFFIX" --baseline "$O/baseline-boundary$SUFFIX" "${ARGS[@]}"
 done
 python3 "$A/appended_question.py" "$B" "$O/baseline-stale"
 python3 "$A/appended_question.py" "$C" "$O/candidate-stale"
 ;;
full)
 python3 "$A/story_dev.py" "$B" "$C" "$O/story-sample" --model "$M" --sample 20
 python3 "$A/story_dev.py" "$B" "$C" "$O/story-full" --model "$M"
 python3 ../coordinator/socialrouting-review/intent-routing.py "$C" "$O/intent-full" --baseline ../coordinator/socialrouting-review/intent-baseline
 ;;
regressions)
 python3 tools/talk-dev.py "$C" --model "$M" --output "$O/calc27.json"
 python3 tools/talk-story-dev.py "$C" --model "$M" --output "$O/story47.json"
 python3 tools/facts/sets/contracts.py --binary "$C" --output "$O/facts"
 python3 tools/facts/sets/session_contracts.py --binary "$C" --bundle "$O/facts/bundle" --output "$O/facts-session"
 ;;
esac
