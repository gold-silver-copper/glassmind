#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
capped() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
capped cargo fmt -p glassmind-quantities
capped cargo fmt -p glassmind-quantities -- --check
capped cargo clippy -p glassmind-quantities --all-targets -j1 -- -D warnings
capped cargo test -p glassmind-quantities -j1 -- --test-threads 4
capped cargo build -p glassmind-quantities -j1
