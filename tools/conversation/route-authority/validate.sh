#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/scratch
OUT=${ROUTE_REPORTS:-tools/conversation/route-authority/reports}
mkdir -p "$OUT"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt --all > "$OUT/fmt.log" 2>&1
sha256sum crates/cli/src/{everyday,talk_stories}.rs > "$OUT/source.sha256"
git diff -- crates/cli/src/everyday.rs crates/cli/src/talk_stories.rs > "$OUT/runtime.patch"
cap cargo build -p glassmind --release -j1 > "$OUT/build.log" 2>&1
cp "$CARGO_TARGET_DIR/release/glassmind" ../routeauthority-run/glassmind-candidate
sha256sum ../routeauthority-run/glassmind-candidate > "$OUT/binary.sha256"
cap cargo test -p glassmind --release everyday::tests -j1 -- --test-threads 4 > "$OUT/test-everyday.log" 2>&1
cap cargo test -p glassmind --release talk_stories::tests -j1 -- --test-threads 4 > "$OUT/test-stories.log" 2>&1
cap cargo test -p glassmind-everyday --release --lib -j1 -- --test-threads 4 > "$OUT/test-memory.log" 2>&1
cap cargo clippy -p glassmind -p glassmind-everyday --all-targets -j1 -- -D warnings > "$OUT/clippy.log" 2>&1
cap cargo fmt --all -- --check > "$OUT/fmt-check.log" 2>&1
git diff --check
