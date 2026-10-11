#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
OUT=tools/conversation/social-routing/reports
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt --all > "$OUT/fmt-final.log" 2>&1
cap cargo build -p glassmind --release -j1 > "$OUT/build-final.log" 2>&1
cap cargo test -p glassmind --release everyday::tests -j1 -- --test-threads 4 > "$OUT/test-cli.log" 2>&1
cap cargo test -p glassmind --release talk_stories::tests -j1 -- --test-threads 4 > "$OUT/test-stories.log" 2>&1
cap cargo test -p glassmind-everyday --release --lib -j1 -- --test-threads 4 > "$OUT/test-memory.log" 2>&1
cap cargo clippy -p glassmind -p glassmind-everyday --all-targets -j1 -- -D warnings > "$OUT/clippy.log" 2>&1
cap cargo fmt --all -- --check > "$OUT/fmt-check.log" 2>&1
git diff --check > "$OUT/diff-check.log"
