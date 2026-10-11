#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
RUN=/home/gsc/.cache/glassmind/continuous-20261010/percentchange-run
export TMPDIR="$RUN/tmp" GLASSMIND_SCRATCH="$RUN/scratch"
export CARGO_TARGET_DIR="$RUN/target"
capped() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
capped cargo fmt -p glassmind-quantities
capped cargo fmt -p glassmind-quantities -- --check
capped cargo clippy -p glassmind-quantities --all-targets -j1 -- -D warnings
capped cargo test -p glassmind-quantities -j1 -- --test-threads 4
capped cargo build -p glassmind-quantities -p glassmind -j1
