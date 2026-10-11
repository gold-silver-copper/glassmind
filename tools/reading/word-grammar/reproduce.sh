#!/usr/bin/env bash
# Use a fresh output directory: completed model/results are never overwritten.
set -euo pipefail
: "${CARGO_TARGET_DIR:?Use the existing target directory}"
: "${TMPDIR:?Set a non-/tmp temporary directory}"
: "${WORD_RUN:?Set a new output directory}"
BASE=/home/gsc/.cache/glassmind/continuous-20261010/reading-run/probability
DEV=/home/gsc/.cache/glassmind/grow-fast-20261010/reading-data/grammar.jsonl
mkdir -p "$WORD_RUN"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-reading --check
cap cargo clippy -p glassmind-reading --bin reading-word-probability --tests -j1 -- -D warnings
cap cargo test -p glassmind-reading --bin reading-word-probability -j1 -- --test-threads 4
cap cargo build --release -p glassmind-reading --bin reading-word-probability -j1
python3 tools/reading/word-grammar/freeze.py verify "$WORD_RUN"
cap python3 tools/reading/measure.py --output "$WORD_RUN/train-resources.json" -- \
  "$CARGO_TARGET_DIR/release/reading-word-probability" train --model "$WORD_RUN/word3.bin" --input \
  "$BASE/bnc_spoken.train" "$BASE/childes.train" "$BASE/gutenberg.train" \
  "$BASE/open_subtitles.train" "$BASE/simple_wiki.train" "$BASE/switchboard.train" > "$WORD_RUN/train.json"
python3 tools/reading/word-grammar/freeze.py freeze "$WORD_RUN"
for split in sample full; do
  input="$DEV"
  if [[ "$split" == sample ]]; then input="$WORD_RUN/grammar-sample.jsonl"; fi
  cap python3 tools/reading/measure.py --output "$WORD_RUN/$split-resources.json" -- \
    "$WORD_RUN/reading-word-probability" evaluate --model "$WORD_RUN/word3.bin" \
    --baseline "$BASE/byte6.bin" --input "$input" > "$WORD_RUN/$split-costs.jsonl"
  extra=()
  if [[ "$split" == full ]]; then extra+=(--full); fi
  python3 tools/reading/word-grammar/audit.py --input "$input" --scores "$WORD_RUN/$split-costs.jsonl" \
    --baseline-report "$BASE/grammar-full.json" --output "$WORD_RUN/$split" "${extra[@]}"
done
