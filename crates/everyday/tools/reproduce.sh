#!/usr/bin/env bash
# Run from any directory. All generated files stay under scratch/everyday.
set -euo pipefail
cd "$(dirname "$0")/../../.."
work="$PWD/scratch/everyday"
mkdir -p "$work/tmp"
export TMPDIR="$work/tmp"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
cap() {
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=5s 600 "$@"
}

cap cargo fmt --manifest-path crates/everyday/Cargo.toml -- --check
cap cargo build -p glassmind-everyday --release -j 1
cap cargo clippy -p glassmind-everyday --all-targets -j 1 -- -D warnings
cap python3 crates/everyday/tools/fetch_clinc.py "$work/clinc"
binary="$CARGO_TARGET_DIR/release/glassmind-everyday"
cap "$binary" train --train "$work/clinc/train.jsonl" --dev "$work/clinc/val.jsonl" \
  --model "$work/centroid.json" --epochs 0 --char-weight 0 --per-label 3 \
  > "$work/centroid-sample.json"
cap "$binary" eval --model "$work/centroid.json" --dev "$work/clinc/val.jsonl" \
  --per-label 0 > "$work/centroid-full-before-calibration.json"
cap "$binary" calibrate --model "$work/centroid.json" --dev "$work/clinc/val.jsonl" \
  --wrong-cost 2 > "$work/centroid-full-calibration.json"
cap "$binary" eval --model "$work/centroid.json" --dev "$work/clinc/val.jsonl" \
  --per-label 0 > "$work/centroid-full.json"
cap "$binary" train --train "$work/clinc/train.jsonl" --dev "$work/clinc/val.jsonl" \
  --model "$work/intent.json" --epochs 20 --char-weight 0.3 --per-label 3 \
  > "$work/margin-sample.json"
cap "$binary" calibrate --model "$work/intent.json" --dev "$work/clinc/val.jsonl" \
  --wrong-cost 2 > "$work/margin-full-calibration.json"
cap "$binary" eval --model "$work/intent.json" --dev "$work/clinc/val.jsonl" \
  --per-label 0 > "$work/margin-full.json"
cap python3 crates/everyday/tools/smoke_memory.py "$binary" "$work/memory-smoke" \
  > "$work/memory-smoke.json"
python3 - "$work" <<'PY'
import json, pathlib, sys
work = pathlib.Path(sys.argv[1])
for name in ['centroid-sample', 'margin-sample', 'centroid-full', 'margin-full', 'memory-smoke']:
    result = json.loads((work / (name + '.json')).read_text())
    print(name, json.dumps(result.get('dev_selection', result), sort_keys=True))
print('Chat:', 'glassmind-everyday chat --model', work / 'intent.json')
PY
