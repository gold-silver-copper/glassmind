#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${TMPDIR:?Set an on-disk TMPDIR outside /tmp}"
: "${CARGO_TARGET_DIR:?Use the existing target directory}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
export TMPDIR CARGO_TARGET_DIR
mkdir -p "$TMPDIR/mathqa"
# Requires pyarrow 23.0.1 in the selected Python environment. On the worker,
# PYTHONPATH="$TMPDIR/quantities-deps" provides the unpacked official wheel.
export PYTHONPATH="${PYTHONPATH:-$TMPDIR/quantities-deps}"
capped() {
    systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=10s 600 "$@"
}
python3 tools/quantities/fetch_mathqa.py --data "$TMPDIR/mathqa"
capped cargo fmt -p glassmind-quantities -- --check
capped cargo clippy -p glassmind-quantities --all-targets -j1 -- -D warnings
capped cargo test -p glassmind-quantities -j1 -- --test-threads 4
capped cargo build -p glassmind-quantities -j1
for sample in sample full; do
    args=()
    if [[ "$sample" == sample ]]; then args+=(--sample); fi
    capped python3 tools/quantities/evaluate_mathqa.py --data "$TMPDIR/mathqa" \
        --binary "$CARGO_TARGET_DIR/debug/glassmind-quantities" "${args[@]}" \
        --report "tools/quantities/reports/final-$sample.json"
done
capped python3 crates/quantities/dev/evaluate.py \
    --report tools/quantities/reports/synthetic-core.json
capped python3 crates/quantities/dev/evaluate.py --suite extended \
    --report tools/quantities/reports/synthetic-extended.json
