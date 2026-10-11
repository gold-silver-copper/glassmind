#!/usr/bin/env bash
# Reproduce the REJECTED experiment in a clean isolated checkout at base38ef238.
set -euo pipefail
: "${TMPDIR:?persistent scratch required}" "${CARGO_TARGET_DIR:?existing target required}"
export PYTHONDONTWRITEBYTECODE=1
work=/home/gsc/.cache/glassmind/continuous-20261010
root=${SGD_ROOT:-$work/dialogue-run/data-sgd-full}
base=${FRESHNESS_BASELINE:-$work/freshness-run/baseline}
run="$TMPDIR/freshness-reproduction"
metrics="$work/dialogue-run/tmp/metrics-venv/bin/python"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
git diff --quiet -- crates/dialogue
git apply --check tools/dialogue/freshness/rejected.patch
git apply tools/dialogue/freshness/rejected.patch
trap 'git apply --reverse tools/dialogue/freshness/rejected.patch' EXIT
cap cargo fmt -p glassmind-dialogue -- --check
cap cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
cap cargo test --release -p glassmind-dialogue --lib -j1 -- --test-threads 4
cap cargo build --release -p glassmind-dialogue -j1
cap python tools/dialogue/freshness/diagnose.py prepare --root "$root" --run "$run" --baseline "$base"
cap "$CARGO_TARGET_DIR/release/dialogue-span-trace" "$base/model.json" "$run/train-inputs.json" "$run/train-traces.json"
cap python tools/dialogue/freshness/diagnose.py report --root "$root" --run "$run" --baseline "$base"
# First100 yielded no changed TRAIN spans. These DEV runs are falsification,
# not a claim that the preregistered TRAIN evidence gate passed.
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$run/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/first" --sample first
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$run/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/full" --audit
cap "$metrics" tools/dialogue/compare.py --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --baseline "$base/full/predictions.json" --candidate "$run/full/predictions.json" --output "$run/transitions.json"
cap python tools/dialogue/freshness/audit.py --data "$root/selected/dev.json" --baseline "$base/full/predictions.json" --candidate "$run/full/predictions.json" --inherited "$base/audit.json" --output "$run/audit.json"
cap "$CARGO_TARGET_DIR/release/dialogue-span-trace" "$base/model.json" "$run/full/inputs.json" "$run/dev-traces.json"
cap python tools/dialogue/freshness/diagnose.py prepare --limit 0 --root "$root" --run "$run/all-train" --baseline "$base"
cap "$CARGO_TARGET_DIR/release/dialogue-span-trace" "$base/model.json" "$run/all-train/train-inputs.json" "$run/all-train/train-traces.json"
cap python tools/dialogue/freshness/diagnose.py report --limit 0 --root "$root" --run "$run/all-train" --baseline "$base"
cap "$metrics" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$base/model.json" --data "$root/selected/dev.json" --schema "$root/selected/dev-schema.json" --output "$run/frozen-replay"
cmp "$run/frozen-replay/predictions.json" "$base/full/predictions.json"
cap "$CARGO_TARGET_DIR/release/glassmind-dialogue" eval --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --model /home/gsc/.cache/glassmind/models/dialogue/model-12.json --output "$run/legacy-predictions.json"
cmp "$run/legacy-predictions.json" "$work/dialogue-run/tmp/multi/final-legacy-predictions.json"
