#!/bin/sh
# One bounded CI/local check, with no full-set evaluations or test suite.
set -eu
: "${TMPDIR:=$HOME/.cache/glassmind/check-tmp}"
: "${CARGO_BUILD_JOBS:=4}"
if [ "$CARGO_BUILD_JOBS" -gt 4 ]; then CARGO_BUILD_JOBS=4; fi
export TMPDIR CARGO_BUILD_JOBS
mkdir -p "$TMPDIR"
exec systemd-run --user --scope --quiet --collect \
  -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 1800 sh -eu -c '
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -j "$CARGO_BUILD_JOBS" -- -D warnings
    cargo build --release --workspace -j "$CARGO_BUILD_JOBS"
    scratch=$(mktemp -d "$TMPDIR/smoke.XXXXXX")
    trap '\''rm -rf "$scratch"'\'' EXIT HUP INT TERM
    mkdir -p "$scratch/sources"
    cp -R fixtures/tiny "$scratch/sources/tiny"
    "${CARGO_TARGET_DIR:-target}/release/glassmind" eval \
      --sources "$scratch/sources" --inputs fixtures/data/train/inputs.jsonl > "$scratch/out.jsonl"
    "${CARGO_TARGET_DIR:-target}/release/glassmind-loop" score \
      --labels fixtures/data/train/labels.jsonl --outcomes "$scratch/out.jsonl"
  '
