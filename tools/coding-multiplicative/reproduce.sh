#!/usr/bin/env bash
set -euo pipefail
# Run from repository root. Use a fresh output directory; never overwrite evidence.
: "${CARGO_TARGET_DIR:?existing target directory required}"
: "${TMPDIR:?persistent non-/tmp directory required}"
: "${BASELINE:?path to frozen original glassmind required}"
: "${OUT:?fresh persistent output directory required}"
TRAIN=${TRAIN:-/home/gsc/.cache/glassmind/data/train-v3}
SOURCES=${SOURCES:-/home/gsc/.cache/glassmind/sources}
mkdir -p "$OUT"
export CARGO_NET_OFFLINE=true
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
sha256sum -c tools/coding-multiplicative/PLAN.sha256
sha256sum -c tools/coding-multiplicative/SUPPLEMENT.sha256
cap cargo fmt -p glassmind-world-rust --check
cap cargo test -j1 --offline -p glassmind-world-rust -- --test-threads 4
cap cargo clippy -j1 --offline -p glassmind-world-rust --all-targets -- -D warnings
cap cargo build -j1 --offline --release -p glassmind
cp "$CARGO_TARGET_DIR/release/glassmind" "$OUT/glassmind"
for mode in baseline candidate; do
  binary="$OUT/glassmind"
  flags=()
  if [[ "$mode" == baseline ]]; then binary="$BASELINE"; flags=(--baseline); fi
  python3 tools/coding-multiplicative/probe.py --binary "$binary" --out "$OUT/$mode-authored" "${flags[@]}"
  python3 tools/coding-multiplicative/probe.py --binary "$binary" --out "$OUT/$mode-supplement" --fixtures tools/coding-multiplicative/supplement.json "${flags[@]}"
  python3 tools/coding-repair-dev.py --binary "$binary" --train "$TRAIN" --sources "$SOURCES" --out "$OUT/$mode-sample" --sample
  python3 tools/coding-repair-dev.py --binary "$binary" --train "$TRAIN" --sources "$SOURCES" --out "$OUT/$mode-full" --check-cache "$OUT/$mode-sample/checks"
done
