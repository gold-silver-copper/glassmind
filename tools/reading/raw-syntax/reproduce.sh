#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?non-/tmp storage required}"
: "${CARGO_TARGET_DIR:?existing target required}"
SOURCE=${SOURCE:-/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source}
BASELINE=${BASELINE:-/home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/baseline}
OUT=${OUT:-/home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/reproduction}
mkdir -p "$OUT"
scoped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
scoped cargo fmt -p glassmind-reading -- --check
scoped cargo test -p glassmind-reading -j1 -- --test-threads 4
scoped cargo clippy -p glassmind-reading --all-targets -j1 -- -D warnings
scoped cargo build --release -p glassmind-reading --bin reading-syntax -j1
cp "$CARGO_TARGET_DIR/release/reading-syntax" "$OUT/reading-syntax"
cp "$BASELINE/model.bin" "$OUT/model.bin"
sha256sum crates/reading/src/syntax.rs "$OUT/reading-syntax" "$OUT/model.bin" \
 tools/reading/raw-syntax/{PLAN.md,authored.jsonl,evaluate.py} > "$OUT/freeze.sha256"
for mode in authored sample full forms; do
 scoped python3 tools/reading/raw-syntax/evaluate.py --source "$SOURCE" --baseline "$BASELINE" \
  --binary "$OUT/reading-syntax" --out "$OUT" --mode "$mode"
done
scoped python3 tools/reading/syntax/contracts.py "$OUT"
TASK_READING_RLIB=$(rg --files "$CARGO_TARGET_DIR/release/deps" -g 'libglassmind_reading-*.rlib' | sort | head -1)
scoped rustc --edition=2024 tools/reading/syntax/model_check.rs \
 -L "dependency=$CARGO_TARGET_DIR/release/deps" --extern "glassmind_reading=$TASK_READING_RLIB" -o "$OUT/model-check"
scoped "$OUT/model-check" "$OUT/model.bin"
