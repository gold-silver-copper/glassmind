#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
set -euo pipefail
: "${CARGO_TARGET_DIR:?Use the coordinator-provided existing target directory}"
: "${TMPDIR:?Use persistent on-disk storage, never /tmp}"
: "${OWNRUN:?Path to retained indexproof-run}"
export GLASSMIND_SCRATCH="$OWNRUN/scratch"
export GLASSMIND_CARGO_TARGET="$CARGO_TARGET_DIR"
capped_cargo() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
capped_cargo fmt --all -- --check
capped_cargo test -p glassmind-world-rust -j1 -- --test-threads 4
capped_cargo clippy -p glassmind-world-rust -j1 -- -D warnings
capped_cargo build -p glassmind -j1
python3 tools/coding-index-proof/run.py --binary "$OWNRUN/baseline/glassmind" --dev-inputs "$OWNRUN/dev-inputs" --out "$TMPDIR/replay-baseline-sample" --baseline --sample
python3 tools/coding-index-proof/run.py --binary "$OWNRUN/baseline/glassmind" --dev-inputs "$OWNRUN/dev-inputs" --out "$TMPDIR/replay-baseline-full" --baseline
python3 tools/coding-index-proof/run.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs "$OWNRUN/dev-inputs" --out "$TMPDIR/replay-sample" --sample
python3 tools/coding-index-proof/run.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs "$OWNRUN/dev-inputs" --out "$TMPDIR/replay-full"
python3 tools/coding-index-proof/width_probe.py --dev-inputs "$OWNRUN/dev-inputs" --out "$TMPDIR/replay-width"
