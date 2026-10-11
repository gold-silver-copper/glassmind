#!/usr/bin/env bash
set -euo pipefail
R=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/buffered-route-review
A=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/submitted/tools/conversation/route-authority
MODE=$1
ARGS=()
SUFFIX=''
if [[ $MODE == model ]]; then ARGS=(--model /home/gsc/.cache/glassmind/models/everyday/intent.json); SUFFIX=-model; fi
B="$R/glassmind-integrated"
python tools/conversation/social-routing/evaluate.py "$B" "$R/social-sample-$MODE" --sample "${ARGS[@]}"
python tools/conversation/social-routing/evaluate.py "$B" "$R/social-full-$MODE" "${ARGS[@]}"
python tools/conversation/memory-authority/evaluate.py "$B" "$R/memory-$MODE" "${ARGS[@]}"
python tools/conversation/memory-authority/learned_social.py "$B" "$R/goodbye-$MODE" "${ARGS[@]}"
python "$A/boundaries.py" "$B" "$R/boundary-sample-$MODE" --sample --baseline "$A/reports/baseline-boundary-sample$SUFFIX" "${ARGS[@]}"
python "$A/boundaries.py" "$B" "$R/boundary-full-$MODE" --baseline "$A/reports/baseline-boundary$SUFFIX" "${ARGS[@]}"
