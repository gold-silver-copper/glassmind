#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${TMPDIR:?Use an on-disk TMPDIR}"
: "${CARGO_TARGET_DIR:?Use the provided existing target directory}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
baseline=$(realpath "${1:?Frozen baseline binary}")
out=$(realpath -m "${2:?New evidence directory}")
mkdir "$out"
capped() {
    TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=10s 600 "$@"
}
# Do not inherit optional glassmind caches into the native contract tests.
unset GLASSMIND_CARGO_TARGET GLASSMIND_CHECK_CACHE
capped python3 tools/coding-check-isolation/run.py --binary "$baseline" --out "$out/baseline-sample" --sample > "$out/baseline-sample.log" 2>&1
capped python3 tools/coding-check-isolation/run.py --binary "$baseline" --out "$out/baseline-full" > "$out/baseline-full.log" 2>&1
capped cargo fmt --all --check > "$out/fmt.log" 2>&1
capped cargo test -p glassmind-world-rust -j1 -- --test-threads 4 > "$out/world-tests.log" 2>&1
capped cargo test -p glassmind -j1 edit::tests -- --test-threads 4 > "$out/cli-edit-tests.log" 2>&1
capped cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings > "$out/clippy.log" 2>&1
capped cargo build -p glassmind -j1 > "$out/build.log" 2>&1
cp "$CARGO_TARGET_DIR/debug/glassmind" "$out/glassmind"
capped python3 tools/coding-check-isolation/run.py --binary "$out/glassmind" --out "$out/candidate-sample" --sample --require-pass > "$out/candidate-sample.log" 2>&1
capped python3 tools/coding-check-isolation/run.py --binary "$out/glassmind" --out "$out/candidate-full" --require-pass > "$out/candidate-full.log" 2>&1
python3 tools/coding-check-isolation/compare.py "$out/baseline-full/results.json" "$out/candidate-full/results.json" "$out/comparison.json"
sha256sum "$baseline" "$out/glassmind" crates/world-rust/src/checks.rs crates/world-rust/tests/check_isolation.rs tools/coding-check-isolation/run.py > "$out/hashes.txt"
