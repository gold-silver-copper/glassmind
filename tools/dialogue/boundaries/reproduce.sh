#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent scratch required}" "${CARGO_TARGET_DIR:?existing target required}"
export PYTHONDONTWRITEBYTECODE=1
work=/home/gsc/.cache/glassmind/continuous-20261010
root=${SGD_ROOT:-$work/dialogue-run/data-sgd-full}
base=${BOUNDARY_BASELINE:-$work/boundaries-run/baseline}
run="$TMPDIR/boundaries"
metrics="$work/dialogue-run/tmp/metrics-venv/bin/python"
mkdir -p "$run"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-dialogue -- --check
cap cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
cap cargo test --release -p glassmind-dialogue --lib -j1 -- --test-threads 4
cap cargo build --release -p glassmind-dialogue -j1
cap python tools/dialogue/boundaries/train.py --train "$root/selected/train.json" --schema "$root/train/schema.json" --baseline "$base/model.json" --output "$run"
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$run/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/first" --sample first
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$run/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/full" --audit
cap "$metrics" tools/dialogue/compare.py --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --baseline "$base/full/predictions.json" --candidate "$run/full/predictions.json" --output "$run/transitions.json"
cap python tools/dialogue/boundaries/audit.py --data "$root/selected/dev.json" --baseline "$base/full/predictions.json" --candidate "$run/full/predictions.json" --inherited "$base/inherited-errors.json" --output "$run/audit.json"
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$base/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/frozen-replay"
cmp "$run/frozen-replay/predictions.json" "$base/full/predictions.json"
cap "$CARGO_TARGET_DIR/release/glassmind-dialogue" eval --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --model /home/gsc/.cache/glassmind/models/dialogue/model-12.json --output "$run/legacy-predictions.json"
cap "$metrics" crates/dialogue/tools/score.py --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --predictions "$run/legacy-predictions.json" --output "$run/legacy.json"
cmp "$run/legacy-predictions.json" "$work/dialogue-run/tmp/multi/final-legacy-predictions.json"
cap python tools/dialogue/boundaries/evidence.py --root "$root" --run "$run" --baseline "$base" --target "$CARGO_TARGET_DIR"
