#!/usr/bin/env bash
# Rebuild/replay the rejected experiment, never candidate DEV.
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${TMPDIR:?set persistent on-disk TMPDIR}"
: "${CARGO_TARGET_DIR:?reuse existing target directory}"
source_dir="${ENTITY_LEDGER_SOURCE:-/home/gsc/.cache/glassmind/continuous-20261010/entityledger-source}"
metrics_python="${ENTITY_LEDGER_METRICS_PYTHON:-/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/metrics-venv/bin/python}"
result_dir="${ENTITY_LEDGER_OUTPUT:-$(mktemp -d "$TMPDIR/entity-ledger-reproduction.XXXXXX")}"
mkdir -p "$result_dir/artifacts" "$result_dir/reports"
export ENTITY_LEDGER_COMMAND_LOG="$result_dir/reports/commands.log"
runner=tools/dialogue/entity-ledger/run.sh
example=crates/dialogue/examples/entity-ledger.rs
module=crates/dialogue/src/entity_ledger_experiment.rs
test ! -e "$example"
test ! -e "$module"
cp tools/dialogue/entity-ledger/native/entity-ledger.rs "$example"
cp tools/dialogue/entity-ledger/native/entity_ledger_experiment.rs "$module"
trap 'rm -f "$example" "$module"' EXIT
bash "$runner" cargo fmt -p glassmind-dialogue -- --check
bash "$runner" cargo test --offline -p glassmind-dialogue --example entity-ledger -j1 -- --test-threads 4
bash "$runner" cargo clippy --offline -p glassmind-dialogue --all-targets -j1 -- -D warnings
bash "$runner" cargo build --offline --release -p glassmind-dialogue --example entity-ledger -j1
cp "$CARGO_TARGET_DIR/release/examples/entity-ledger" "$result_dir/artifacts/entity-ledger-native"
python3 tools/dialogue/entity-ledger/inputs.py --source "$source_dir" --artifacts "$result_dir/artifacts"
bash "$runner" "$source_dir/baseline-dialogue-multi" predict --input "$result_dir/artifacts/train-input.json" --model "$source_dir/model.json" --schema "$source_dir/schema.json" --output "$result_dir/artifacts/baseline-train.json"
python3 tools/dialogue/entity-ledger/prepare.py --source "$source_dir" --artifacts "$result_dir/artifacts"
bash "$runner" "$result_dir/artifacts/entity-ledger-native" "$result_dir/artifacts/supervision.json" "$result_dir/artifacts"
bash "$runner" "$metrics_python" tools/dialogue/entity-ledger/audit.py --artifacts "$result_dir/artifacts" --schema "$source_dir/schema.json"
bash "$runner" python3 tools/dialogue/entity-ledger/review.py --artifacts "$result_dir/artifacts"
python3 - "$result_dir" <<'PY'
import pathlib,json,sys
p=pathlib.Path(sys.argv[1]);r=json.loads((p/'reports/gate.json').read_text());assert r['status']=='REJECTED_CAL';print('REJECTED; raw evidence:',p)
PY
