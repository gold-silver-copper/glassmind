#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
export PYTHONDONTWRITEBYTECODE=1
work=/home/gsc/.cache/glassmind/continuous-20261010
root=$work/dialogue-run/data-sgd-full
run=$TMPDIR/spansequence
base=$work/spansequence-run/baseline
metrics=$work/dialogue-run/tmp/metrics-venv/bin/python
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
python tools/dialogue/spansequence/freeze.py
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$base/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/first" --sample first
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$base/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/full" --audit
cap "$metrics" tools/dialogue/compare.py --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --baseline "$base/full/predictions.json" --candidate "$run/full/predictions.json" --output "$run/transitions.json"
cap python tools/dialogue/spansequence/audit.py --data "$root/selected/dev.json" --baseline "$base/full/predictions.json" --candidate "$run/full/predictions.json" --inherited "$base/audit.json" --output "$run/audit.json"
cmp "$run/first/predictions.json" "$base/first/predictions.json"
cmp "$run/full/predictions.json" "$base/full/predictions.json"
cap "$CARGO_TARGET_DIR/release/glassmind-dialogue" eval --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --model /home/gsc/.cache/glassmind/models/dialogue/model-12.json --output "$run/legacy-predictions.json"
cap "$metrics" crates/dialogue/tools/score.py --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --predictions "$run/legacy-predictions.json" --output "$run/legacy.json"
cmp "$run/legacy-predictions.json" "$work/dialogue-run/tmp/multi/final-legacy-predictions.json"
