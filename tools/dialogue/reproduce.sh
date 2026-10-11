#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
: "${CARGO_TARGET_DIR:?set the existing target directory}"
: "${TMPDIR:?set an on-disk TMPDIR outside /tmp}"
export PYTHONDONTWRITEBYTECODE=1
export SGD_ROOT="${SGD_ROOT:-/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full}"
mkdir -p "$TMPDIR/multi"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap python tools/dialogue/prepare.py > "$TMPDIR/multi/source-selection.json"
if [[ ! -x "$TMPDIR/metrics-venv/bin/python" ]]; then python -m venv "$TMPDIR/metrics-venv"; fi
cap "$TMPDIR/metrics-venv/bin/python" -m pip install --disable-pip-version-check -r crates/dialogue/tools/requirements.txt
cap cargo fmt -p glassmind-dialogue -- --check
cap cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
cap cargo test -p glassmind-dialogue -j1 -- --test-threads 4
cap cargo build -p glassmind-dialogue --release -j1
sgd_data="$SGD_ROOT/selected"
sgd_frozen=/home/gsc/.cache/glassmind/models/dialogue/model-12.json
sgd_run="$TMPDIR/multi"
sgd_binary="$CARGO_TARGET_DIR/release/dialogue-multi"
cap "$sgd_binary" train --data "$sgd_data/train.json" --schema "$sgd_data/train-schema.json" --frozen "$sgd_frozen" --output "$sgd_run/model-12.json" --epochs 12 --router-epochs 12
python - "$sgd_run" <<'PY'
import json,pathlib,sys
p=pathlib.Path(sys.argv[1]);m=json.loads((p/'model-12.json').read_text())
for name,service in m['services'].items():(p/(name+'.json')).write_text(json.dumps(service))
PY
for sgd_variant in centroid independent; do
 sgd_options=(--router-epochs 0)
 if [[ "$sgd_variant" == independent ]]; then sgd_options=(--router-epochs 12 --independent-router); fi
 sgd_output="$sgd_run/model-0.json"
 if [[ "$sgd_variant" == independent ]]; then sgd_output="$sgd_run/model-independent-12.json"; fi
 cap "$sgd_binary" train --data "$sgd_data/train.json" --schema "$sgd_data/train-schema.json" --frozen "$sgd_frozen" --frozen "$sgd_run/Travel_1.json" --frozen "$sgd_run/Weather_1.json" --output "$sgd_output" --epochs 12 "${sgd_options[@]}"
done
bash tools/dialogue/evaluate.sh
for sgd_scope in first full; do
 sgd_options=(--sample first)
 if [[ "$sgd_scope" == full ]]; then sgd_options=(--audit); fi
 cap "$TMPDIR/metrics-venv/bin/python" tools/dialogue/evaluate.py --binary "$sgd_binary" --model "$sgd_run/model-independent-12.json" --data "$sgd_data/dev.json" --schema "$sgd_data/dev-schema.json" --output "$sgd_run/independent-$sgd_scope-12" "${sgd_options[@]}"
done
cap "$CARGO_TARGET_DIR/release/glassmind-dialogue" eval --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --model "$sgd_frozen" --output "$sgd_run/legacy-predictions.json"
cap "$TMPDIR/metrics-venv/bin/python" crates/dialogue/tools/score.py --data /home/gsc/.cache/glassmind/data/sgd/dev.json --schema /home/gsc/.cache/glassmind/data/sgd/dev-schema.json --predictions "$sgd_run/legacy-predictions.json" --output "$sgd_run/legacy-report.json"
for sgd_variant in joint independent; do
 sgd_base="$sgd_run/full-0/predictions.json";sgd_candidate="$sgd_run/full-12/predictions.json"
 if [[ "$sgd_variant" == independent ]]; then sgd_base="$sgd_candidate";sgd_candidate="$sgd_run/independent-full-12/predictions.json";fi
 cap "$TMPDIR/metrics-venv/bin/python" tools/dialogue/compare.py --data "$sgd_data/dev.json" --schema "$sgd_data/dev-schema.json" --baseline "$sgd_base" --candidate "$sgd_candidate" --output "$sgd_run/$sgd_variant-transitions.json"
done
