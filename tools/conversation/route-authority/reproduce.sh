#!/usr/bin/env bash
# Run from worktree root; choose a fresh output directory to preserve old evidence.
set -euo pipefail
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/target
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/scratch
: "${ROUTE_REPORTS:?Set a fresh output directory; do not overwrite retained evidence}"
[[ ! -e "$ROUTE_REPORTS" ]]
mkdir -p "$ROUTE_REPORTS"
sha256sum -c tools/conversation/route-authority/FROZEN.sha256
B=../routeauthority-run/baseline/glassmind
C=../routeauthority-run/glassmind-candidate
M=/home/gsc/.cache/glassmind/models/everyday/intent.json
for MODE in plain model; do
 ARGS=(); SUFFIX=''; if [[ $MODE == model ]]; then ARGS=(--model "$M"); SUFFIX=-model; fi
 python3 tools/conversation/route-authority/boundaries.py "$B" "$ROUTE_REPORTS/baseline-boundary-sample$SUFFIX" --sample "${ARGS[@]}"
 python3 tools/conversation/route-authority/boundaries.py "$B" "$ROUTE_REPORTS/baseline-boundary$SUFFIX" "${ARGS[@]}"
done
python3 ../socialrouting-run/submitted/tools/conversation/social-routing/evaluate.py "$B" "$ROUTE_REPORTS/baseline-old-sample-model" --sample --model "$M"
bash tools/conversation/route-authority/validate.sh
python3 tools/conversation/route-authority/boundaries.py "$C" "$ROUTE_REPORTS/candidate-boundary-sample" --sample --baseline "$ROUTE_REPORTS/baseline-boundary-sample"
for group in boundaries plain model full regressions; do
 bash tools/conversation/route-authority/gates.sh "$group"
done
python3 tools/conversation/route-authority/audit.py
python3 tools/conversation/route-authority/check_gates.py
