#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${TMPDIR:?supply the owned on-disk TMPDIR}"
: "${CARGO_TARGET_DIR:?supply the existing isolated target directory}"
: "${REPAIR_RUN:?supply a fresh output directory outside /tmp}"
: "${BASELINE_BINARY:?supply the frozen bb89964 glassmind binary}"
export TMPDIR CARGO_TARGET_DIR
bounded_cargo() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
mkdir -p "$REPAIR_RUN"
sha256sum -c tools/coding-repair-speech/FROZEN.sha256
bounded_cargo fmt --all --check
bounded_cargo test -p glassmind -p glassmind-english-rust -j1 -- --test-threads 4
bounded_cargo clippy -p glassmind -p glassmind-english-rust --all-targets -j1 -- -D warnings
bounded_cargo build -p glassmind -j1
cp "$CARGO_TARGET_DIR/debug/glassmind" "$REPAIR_RUN/glassmind"
sha256sum "$REPAIR_RUN/glassmind"
python3 tools/coding-repair-speech/probe.py "$BASELINE_BINARY" "$REPAIR_RUN/baseline-sample" --limit 3 --native
python3 tools/coding-repair-speech/probe.py "$BASELINE_BINARY" "$REPAIR_RUN/baseline-full"
python3 tools/coding-repair-speech/probe.py "$REPAIR_RUN/glassmind" "$REPAIR_RUN/sample" --limit 3 --native
python3 tools/coding-repair-speech/probe.py "$REPAIR_RUN/glassmind" "$REPAIR_RUN/full" --native
python3 tools/coding-repair-speech/probe.py "$REPAIR_RUN/glassmind" "$REPAIR_RUN/supplied" --cases tools/coding-repair-speech/supplied.tsv --native
python3 tools/coding-repair-speech/probe.py "$REPAIR_RUN/glassmind" "$REPAIR_RUN/legacy" --cases tools/coding-repair-speech/public-legacy.tsv --native
python3 tools/coding-repair-speech/confirm.py "$REPAIR_RUN/glassmind" "$REPAIR_RUN/confirm"
python3 tools/coding-repair-speech/summarize.py "$REPAIR_RUN/full/results.json"
# Raw mutation_ok/native_ok deliberately retain the frozen incorrect expectation
# that initial ask --apply applies a newly proposed repair. These failures remain
# failures; confirm.py separately tests the actual affirmative session protocol.
