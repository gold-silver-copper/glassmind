#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?set persistent worker TMPDIR}" "${CARGO_TARGET_DIR:?use existing worker target}"
export PYTHONDONTWRITEBYTECODE=1
sgd_root="${SGD_ROOT:-/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full}"
sgd_baseline=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/multi
sgd_python=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/metrics-venv/bin/python
sgd_run="$TMPDIR/references"
mkdir -p "$sgd_run"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-dialogue -- --check
cap cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
cap cargo test --release -p glassmind-dialogue --lib -j1 -- --test-threads 4
cap cargo build --release -p glassmind-dialogue -j1
cap python tools/dialogue/references/train.py --train "$sgd_root/selected/train.json" --schema "$sgd_root/selected/train-schema.json" --output "$sgd_run/relations.json"
cap python tools/dialogue/references/assemble.py --baseline "$sgd_baseline/model-12.json" --relations "$sgd_run/relations.json" --output "$sgd_run/model.json"
cap "$sgd_python" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$sgd_run/model.json" --data "$sgd_root/selected/dev.json" --schema "$sgd_root/selected/dev-schema.json" --output "$sgd_run/first" --sample first
cap "$sgd_python" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$sgd_run/model.json" --data "$sgd_root/selected/dev.json" --schema "$sgd_root/selected/dev-schema.json" --output "$sgd_run/full" --audit
cap "$sgd_python" tools/dialogue/compare.py --data "$sgd_root/selected/dev.json" --schema "$sgd_root/selected/dev-schema.json" --baseline "$sgd_baseline/full-12/predictions.json" --candidate "$sgd_run/full/predictions.json" --output "$sgd_run/transitions.json"
cap python tools/dialogue/references/audit.py --data "$sgd_root/selected/dev.json" --baseline "$sgd_baseline/full-12/predictions.json" --candidate "$sgd_run/full/predictions.json" --model "$sgd_run/model.json" --output "$sgd_run/copy-audit.json"
# The pre-existing legacy cache/model are separate from the full-SGD mount.
cap "$CARGO_TARGET_DIR/release/glassmind-dialogue" eval --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --model /home/gsc/.cache/glassmind/models/dialogue/model-12.json --output "$sgd_run/legacy-predictions.json"
cap "$sgd_python" crates/dialogue/tools/score.py --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --predictions "$sgd_run/legacy-predictions.json" --output "$sgd_run/legacy-report.json"
cmp "$sgd_run/legacy-predictions.json" "$sgd_baseline/final-legacy-predictions.json"
cap python tools/dialogue/references/train_diagnostic.py --train "$sgd_root/selected/train.json" --schema "$sgd_root/selected/train-schema.json" --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --baseline "$sgd_baseline/model-12.json" --relations "$sgd_run/relations.json" --output "$sgd_run/train-diagnostic"
cap "$sgd_python" tools/dialogue/evaluate.py --binary "$CARGO_TARGET_DIR/release/dialogue-multi" --model "$sgd_baseline/model-12.json" --data "$sgd_root/selected/dev.json" --schema "$sgd_root/selected/dev-schema.json" --output "$sgd_run/baseline-replay"
cmp "$sgd_run/baseline-replay/predictions.json" "$sgd_baseline/full-12/predictions.json"
