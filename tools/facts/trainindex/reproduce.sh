#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?Set persistent owned TMPDIR outside /tmp}"
: "${CARGO_TARGET_DIR:?Use existing target directory}"
SOURCE=${SOURCE:-/home/gsc/.cache/glassmind/data/simplequestions}
RUN=${RUN:-$TMPDIR/trainindex}
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
python3 tools/facts/trainindex/prepare.py select --source "$SOURCE" --output "$RUN"
# Explicit opt-in only; fetch rejects existing ledger to prevent budget resets.
if [[ ${FETCH:-0} == 1 ]]; then
  timeout --kill-after=10s 600 python3 tools/facts/trainindex/prepare.py fetch --source "$SOURCE" --output "$RUN"
fi
python3 tools/facts/trainindex/prepare.py rebuild --source "$SOURCE" --output "$RUN"
cap cargo build --release -p glassmind -p glassmind-facts --bins --example continuous_predictions -j1
TMPDIR="$TMPDIR" python3 -m unittest discover -s tools/facts/trainindex -p test_prepare.py
for size in 100 1000; do
  python3 tools/facts/chat/package.py --index "$RUN/scope$size/index.json" --labels "$RUN/scope$size/labels.json" --model "$SOURCE/relation.json" --source 'Wikidata actual cached claims; TRAIN subject-selected series v1' --license CC0-1.0 --scope "TRAIN prefix90 fit frequency top$size; finite snapshot; see rebuild.json for actual coverage" --output "$RUN/bundle$size"
done
python3 tools/facts/trainindex/check_bundle.py --run "$RUN" --binary "$CARGO_TARGET_DIR/release/glassmind"
python3 tools/facts/trainindex/evaluate.py --run "$RUN" --target "$CARGO_TARGET_DIR" --valid "$SOURCE/valid.tsv" --limit 200
python3 tools/facts/trainindex/evaluate.py --run "$RUN" --target "$CARGO_TARGET_DIR" --valid "$SOURCE/valid.tsv"
