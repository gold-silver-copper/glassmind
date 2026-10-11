#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}" "${GLASSMIND_SCRATCH:?}"
run=${TMPDIR%/tmp}
scope=(systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600)
cp crates/world-rust/src/lib.rs "$run/lib-final.rs"
trap 'cp "$run/lib-final.rs" crates/world-rust/src/lib.rs' EXIT
git show 5f2859a730f6d20a0ac64ca9a7551821ed123e86:crates/world-rust/src/lib.rs > crates/world-rust/src/lib.rs
"${scope[@]}" cargo test -p glassmind-world-rust -j1 -- --test-threads 4 > "$run/world-baseline.log" 2>&1
cp "$run/lib-final.rs" crates/world-rust/src/lib.rs
"${scope[@]}" cargo test -p glassmind-world-rust -j1 -- --test-threads 4 > "$run/world-final.log" 2>&1
"${scope[@]}" cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings > "$run/clippy.log" 2>&1
"${scope[@]}" cargo fmt -p glassmind-world-rust -- --check > "$run/fmt.log" 2>&1
