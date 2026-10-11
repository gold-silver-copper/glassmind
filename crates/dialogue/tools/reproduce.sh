#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../../.."
sgd_artifacts="$PWD/scratch/dialogue"
mkdir -p "$sgd_artifacts/tmp" "$sgd_artifacts/reports"
export TMPDIR="$sgd_artifacts/tmp"
export PYTHONDONTWRITEBYTECODE=1
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$sgd_artifacts/target}"
cap() {
    TMPDIR="$sgd_artifacts/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 "$@"
}
cap python3 crates/dialogue/tools/fetch_sgd.py "$sgd_artifacts/rides"
python3 -m venv "$sgd_artifacts/venv"
cap "$sgd_artifacts/venv/bin/python" -m pip install --disable-pip-version-check -r crates/dialogue/tools/requirements.txt
TMPDIR="$sgd_artifacts/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 cargo fmt -p glassmind-dialogue -- --check
TMPDIR="$sgd_artifacts/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 cargo clippy -p glassmind-dialogue --all-targets -j 1 -- -D warnings
TMPDIR="$sgd_artifacts/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 cargo build -p glassmind-dialogue --release -j 1
sgd_binary="$CARGO_TARGET_DIR/release/glassmind-dialogue"
for sgd_epochs in 0 12; do
    cap "$sgd_binary" train --data "$sgd_artifacts/rides/train.json" --schema "$sgd_artifacts/rides/train-schema.json" --output "$sgd_artifacts/model-$sgd_epochs.json" --epochs "$sgd_epochs"
done
evaluate() {
    local name="$1" model="$2"
    shift 2
    cap "$sgd_binary" eval --data "$sgd_artifacts/rides/dev.json" --schema "$sgd_artifacts/rides/dev-schema.json" --model "$sgd_artifacts/model-$model.json" --output "$sgd_artifacts/$name-predictions.json" "$@"
    local limits=()
    if [[ "$name" == sample-* ]]; then limits=(--limit 10); fi
    cap "$sgd_artifacts/venv/bin/python" crates/dialogue/tools/score.py --data "$sgd_artifacts/rides/dev.json" --schema "$sgd_artifacts/rides/dev-schema.json" --predictions "$sgd_artifacts/$name-predictions.json" --output "$sgd_artifacts/reports/$name.json" "${limits[@]}"
}
evaluate sample-centroid 0 --limit 10
evaluate sample-trained 12 --limit 10
evaluate sample-no-carry 12 --limit 10 --no-carry
evaluate dev-centroid 0
evaluate dev-trained 12
evaluate dev-no-carry 12 --no-carry
cap python3 crates/dialogue/tools/audit_runtime.py --binary "$sgd_binary" --data "$sgd_artifacts/rides/dev.json" --schema "$sgd_artifacts/rides/dev-schema.json" --model "$sgd_artifacts/model-12.json" --predictions "$sgd_artifacts/dev-trained-predictions.json" --scratch "$sgd_artifacts/audit"
