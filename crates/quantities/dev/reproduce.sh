#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
export TMPDIR="$PWD/.quantities-tmp"
export RUST_TEST_THREADS=4
mkdir -p "$TMPDIR"

# --wait keeps setsid's parent alive when it forks, retaining scope supervision.
capped() {
    systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=5s 600 "$@"
}

capped cargo fmt -p glassmind-quantities -- --check
capped cargo clippy -p glassmind-quantities --all-targets -j 1 -- -D warnings
capped cargo build -p glassmind-quantities -j 1
capped python3 crates/quantities/dev/evaluate.py --sample --report crates/quantities/dev/final-sample.json
capped python3 crates/quantities/dev/evaluate.py --suite extended --sample --report crates/quantities/dev/extended-sample.json
capped python3 crates/quantities/dev/evaluate.py --report crates/quantities/dev/full-dev.json
capped python3 crates/quantities/dev/evaluate.py --suite extended --report crates/quantities/dev/extended-full-dev.json
capped python3 crates/quantities/dev/aggregate.py
