#!/usr/bin/env bash
set -euo pipefail
# Run from repository root. Reuses the coordinator's existing Cargo target.
: "${CARGO_TARGET_DIR:?existing target required}"
: "${TMPDIR:?external non-/tmp directory required}"
SOURCE=${SOURCE:-/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source}
OUT=${OUT:-/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/reproduction}
mkdir -p "$OUT"
scoped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
scoped cargo fmt --all -- --check
scoped cargo test -p glassmind-reading -j1 -- --test-threads 4
scoped cargo clippy --workspace --all-targets -j1 -- -D warnings
scoped cargo build --release -p glassmind-reading --bin reading-syntax -j1
cp "$CARGO_TARGET_DIR/release/reading-syntax" "$OUT/reading-syntax"
scoped python3 tools/reading/syntax/measure.py "$OUT/train-time.txt" "$OUT/reading-syntax" train \
  --train "$SOURCE/en_ewt-ud-train.conllu" --model "$OUT/model.bin" --report "$OUT/train-report.json"
sha256sum "$OUT/reading-syntax" "$OUT/model.bin" crates/reading/src/syntax.rs \
  crates/reading/src/bin/reading-syntax.rs crates/reading/src/lib.rs tools/reading/syntax/PLAN.md \
  tools/reading/syntax/authored.jsonl tools/reading/syntax/evaluate.py > "$OUT/freeze.sha256"
for mode in sample full authored; do
  python3 tools/reading/syntax/evaluate.py --source "$SOURCE" --artifacts "$OUT" \
    --binary "$OUT/reading-syntax" --model "$OUT/model.bin" --mode "$mode"
done
# Post-freeze contracts; these do not select or retune the model.
PYTHONDONTWRITEBYTECODE=1 python3 tools/reading/syntax/audit_train.py "$SOURCE" "$OUT"
scoped python3 tools/reading/syntax/contracts.py "$OUT"
TASK_READING_RLIB=$(rg --files "$CARGO_TARGET_DIR/release/deps" -g 'libglassmind_reading-*.rlib' | sort | head -1)
scoped rustc --edition=2024 tools/reading/syntax/model_check.rs \
  -L "dependency=$CARGO_TARGET_DIR/release/deps" --extern "glassmind_reading=$TASK_READING_RLIB" \
  -o "$OUT/model-check"
scoped "$OUT/model-check" "$OUT/model.bin"
scoped cargo build --release --workspace -j1
scoped "$CARGO_TARGET_DIR/release/reading-syntax" predict --model "$OUT/model.bin" \
  --input "$OUT/full-input.jsonl" --output "$OUT/workspace-predictions.jsonl"
cmp "$OUT/full-predictions.jsonl" "$OUT/workspace-predictions.jsonl"
