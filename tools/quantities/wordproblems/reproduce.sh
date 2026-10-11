#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
: "${TMPDIR:?Set owned on-disk temporary directory}"
: "${CARGO_TARGET_DIR:?Use existing target directory}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
export TMPDIR CARGO_TARGET_DIR
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
DATA=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa
capped() {
    systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=10s 600 "$@"
}
capped cargo fmt -p glassmind-quantities -- --check
capped cargo clippy -p glassmind-quantities --all-targets -j1 -- -D warnings
capped cargo test -p glassmind-quantities -j1 -- --test-threads 4
capped cargo build -p glassmind-quantities -j1
for version in baseline candidate; do
    binary="$TMPDIR/glassmind-quantities-baseline"
    if [[ "$version" == candidate ]]; then binary="$CARGO_TARGET_DIR/debug/glassmind-quantities"; fi
    capped python3 tools/quantities/evaluate_mathqa.py --data "$DATA" --binary "$binary" \
        --sample --report "$TMPDIR/$version-sample.json"
    capped python3 tools/quantities/evaluate_mathqa.py --data "$DATA" --binary "$binary" \
        --report "$TMPDIR/$version-full.json"
done
capped python3 tools/quantities/wordproblems/compare.py
