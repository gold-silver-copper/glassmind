#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?Set persistent on-disk TMPDIR}" "${CARGO_TARGET_DIR:?Use supplied target}"
[[ "$TMPDIR" != /tmp* ]]
run=${TMPDIR%/tmp}
out=${1:-$run/replay}
baseline=${2:-$run/baseline/glassmind}
mkdir -p "$out"
export GLASSMIND_SCRATCH="$out/scratch"
export CARGO_NET_OFFLINE=true
scope=(systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600)
sha256sum -c tools/coding-field-owners/FROZEN.sha256
"${scope[@]}" cargo fmt -p glassmind-world-rust -- --check
"${scope[@]}" cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
"${scope[@]}" cargo test -p glassmind-world-rust -j1 -- --test-threads 4
"${scope[@]}" cargo build -p glassmind -j1
cp "$CARGO_TARGET_DIR/debug/glassmind" "$out/glassmind-accepted"
for mode in baseline accepted; do
  binary=$baseline
  if [[ "$mode" == accepted ]]; then binary="$out/glassmind-accepted"; fi
  python tools/coding-field-owners/evaluate.py --binary "$binary" --output "$out/$mode-sample" --sample
  python tools/coding-field-owners/evaluate.py --binary "$binary" --output "$out/$mode-full"
  python tools/coding-field-owners/audit.py --binary "$binary" --output "$out/$mode-audit"
  bash tools/coding-field-owners/regressions.sh "$binary" "$out/regressions-$mode"
done
python tools/coding-field-owners/summarize.py "$out" "$out/reports"
