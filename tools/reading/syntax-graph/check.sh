#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
capped() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
capped cargo fmt -p glassmind-reading
capped cargo test -p glassmind-reading -j1 --lib syntax -- --test-threads 4
capped cargo clippy -p glassmind-reading --all-targets -j1 -- -D warnings
capped cargo build -p glassmind-reading --bin reading-syntax --release -j1
capped cargo fmt -p glassmind-reading -- --check
