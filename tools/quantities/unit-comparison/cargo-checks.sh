#!/usr/bin/env bash
set -euo pipefail
: "${CARGO_TARGET_DIR:?existing target required}"
: "${TMPDIR:?persistent tmp required}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
run_cargo() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
case "${1:-test}" in
  test) run_cargo test --offline -p glassmind-quantities -j1 -- --test-threads 4;;
  fmt) run_cargo fmt -p glassmind-quantities -- --check;;
  format) run_cargo fmt -p glassmind-quantities;;
  clippy) run_cargo clippy --offline -p glassmind-quantities --all-targets -j1 -- -D warnings;;
  release) run_cargo build --offline --release -p glassmind-quantities --all-targets -j1;;
  *) exit 2;;
esac
