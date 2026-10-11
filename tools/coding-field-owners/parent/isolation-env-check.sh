#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/isolation-test-env/tmp
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/isolation-test-env/scratch
export CARGO_TARGET_DIR=/home/gsc/code/glassmind/target
export CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true
cargo fmt --all -- --check
cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
cargo test -p glassmind-world-rust --test check_isolation -j1 -- --test-threads 4
