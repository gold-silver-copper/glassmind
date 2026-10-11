#!/usr/bin/env bash
set -euo pipefail
# Run from the ceildivision worktree; no providers, downloads or background jobs.
RUN=/home/gsc/.cache/glassmind/continuous-20261010/ceildivision-run
export TMPDIR="$RUN/tmp"
export CARGO_TARGET_DIR="$RUN/target"
mkdir -p "$TMPDIR"
bounded() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
bounded cargo fmt -p glassmind-world-rust --check
bounded cargo test -p glassmind-world-rust -j1 -- --test-threads 4
bounded cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
bounded cargo build --release -p glassmind -j1
for variant in before after; do
  binary="$RUN/baseline/glassmind"
  if [[ "$variant" == after ]]; then binary="$CARGO_TARGET_DIR/release/glassmind"; fi
  bounded python tools/coding-ceiling-division/authored.py "$binary" "$RUN/repro-authored-$variant"
  bounded python tools/coding-repair-dev.py --binary "$binary" --train /home/gsc/.cache/glassmind/data/train-v3 --sources /home/gsc/.cache/glassmind/sources --out "$RUN/repro-train-sample-$variant" --sample
  bounded python tools/coding-repair-dev.py --binary "$binary" --train /home/gsc/.cache/glassmind/data/train-v3 --sources /home/gsc/.cache/glassmind/sources --out "$RUN/repro-train-full-$variant" --check-cache "$RUN/repro-train-sample-$variant/checks"
done
