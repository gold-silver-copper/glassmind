#!/usr/bin/env bash
set -euo pipefail
WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
RUN_ROOT="${RUN_ROOT:-$(dirname "$WORKTREE")/renameenglish-run}"
REPLAY_ROOT="${REPLAY_ROOT:-$RUN_ROOT/replay}"
export TMPDIR="${TMPDIR:-$RUN_ROOT/tmp}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$RUN_ROOT/target}"
[[ "$TMPDIR" != /tmp && "$TMPDIR" != /tmp/* ]]
mkdir -p "$TMPDIR" "$REPLAY_ROOT"
cd "$WORKTREE"
bounded() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
bounded cargo fmt -p glassmind -p glassmind-english-rust --check
bounded cargo test --offline -p glassmind-english-rust -p glassmind -j1 -- --test-threads 4
bounded cargo clippy --offline -p glassmind-english-rust -p glassmind --all-targets -j1 -- -D warnings
bounded cargo build --offline -p glassmind -j1
bounded python3 tools/coding-rename-language/run.py --binary "$RUN_ROOT/baseline/glassmind" --output "$REPLAY_ROOT/baseline-sample" --supplied "$RUN_ROOT/dev-inputs" --sample
bounded python3 tools/coding-rename-language/run.py --binary "$RUN_ROOT/baseline/glassmind" --output "$REPLAY_ROOT/baseline-full" --supplied "$RUN_ROOT/dev-inputs"
bounded python3 tools/coding-rename-language/run.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$REPLAY_ROOT/candidate-sample" --supplied "$RUN_ROOT/dev-inputs" --sample --both
bounded python3 tools/coding-rename-language/run.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$REPLAY_ROOT/candidate-full" --supplied "$RUN_ROOT/dev-inputs" --both
bounded python3 tools/coding-rename-language/summarize.py "$REPLAY_ROOT/baseline-full/results.json" "$REPLAY_ROOT/candidate-full/results.json" "$REPLAY_ROOT/comparison.json"
bounded python3 tools/coding-rename-language/native.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$REPLAY_ROOT/native"
bounded python3 tools/coding-edits-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$REPLAY_ROOT/edits-regression"
