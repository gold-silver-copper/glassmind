#!/usr/bin/env bash
set -euo pipefail
: "${CARGO_TARGET_DIR:?Use the existing worker target directory}"
: "${TMPDIR:?Use the worker on-disk temporary directory outside /tmp}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
READING_CACHE=${READING_CACHE:-/home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability}
READING_DEV=${READING_DEV:-/home/gsc/.cache/glassmind/grow-fast-20261010/reading-data}
mkdir -p "$READING_CACHE"
limited() {
    systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=10s 600 "$@"
}
limited cargo fmt -p glassmind-reading -- --check
limited cargo test -p glassmind-reading --bin reading-probability -j1 -- --test-threads 4
limited cargo clippy -p glassmind-reading --all-targets -j1 -- -D warnings
limited cargo build -p glassmind-reading --bin reading-probability --release -j1
limited python3 tools/reading/prepare.py --cache "$READING_CACHE" --dev "$READING_DEV"
limited python3 tools/reading/measure.py --output "$READING_CACHE/train-resources.json" \
    "$CARGO_TARGET_DIR/release/reading-probability" train \
    --model "$READING_CACHE/byte6.bin" --input \
    "$READING_CACHE/bnc_spoken.train" "$READING_CACHE/childes.train" \
    "$READING_CACHE/gutenberg.train" "$READING_CACHE/open_subtitles.train" \
    "$READING_CACHE/simple_wiki.train" "$READING_CACHE/switchboard.train" \
    > "$READING_CACHE/train.json"
for subset in sample full; do
    for track in grammar reading; do
        input="$READING_DEV/$track.jsonl"
        if [[ "$subset" == sample ]]; then input="$READING_CACHE/$track-sample.jsonl"; fi
        limited python3 tools/reading/measure.py --output "$READING_CACHE/$track-$subset-resources.json" \
            "$CARGO_TARGET_DIR/release/reading-probability" evaluate \
            --model "$READING_CACHE/byte6.bin" --input "$input" \
            > "$READING_CACHE/$track-$subset.json"
    done
done
limited python3 tools/reading/audit-overlap.py --cache "$READING_CACHE" \
    --dev "$READING_DEV/reading.jsonl" > "$READING_CACHE/overlap.json"
sha256sum "$READING_CACHE/byte6.bin" "$CARGO_TARGET_DIR/release/reading-probability"
