#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?Set TMPDIR to your persistent owned run directory, outside /tmp}"
: "${CARGO_TARGET_DIR:?Use the existing target directory}"
FACTS_DATA=${FACTS_DATA:-/home/gsc/.cache/glassmind/data/simplequestions}
INTENT_MODEL=${INTENT_MODEL:-/home/gsc/.cache/glassmind/models/everyday/intent.json}

cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo build --release -p glassmind -p glassmind-facts --bins --example continuous_predictions -j1
cap cargo test -p glassmind -j1 -- --test-threads 4
cap cargo clippy -p glassmind --all-targets -j1 -- -D warnings
cap cargo fmt -p glassmind --check
# Reuse downloaded raw labels without network. Omit --fetch for normal reproduction.
python3 tools/facts/chat/labels.py --index tools/facts/index.json --cache "$FACTS_DATA/wikidata" --output "$TMPDIR/labels"
python3 tools/facts/chat/package.py --index tools/facts/index.json --model "$FACTS_DATA/relation.json" \
    --labels "$TMPDIR/labels/labels.json" --output "$TMPDIR/bundle" \
    --source 'Wikidata snapshot 2026-10-10; relation model trained on SimpleQuestions df69267' \
    --license 'Wikidata CC0-1.0; SimpleQuestions CC-BY-3.0' \
    --scope '500 validation-subject-seeded entities; finite demonstration, not open-world retrieval'
BIN="$CARGO_TARGET_DIR/release/glassmind"
python3 tools/facts/chat/evaluate.py --binary "$BIN" --facts-binary "$CARGO_TARGET_DIR/release/glassmind-facts" --predictor "$CARGO_TARGET_DIR/release/examples/continuous_predictions" --facts-dir "$TMPDIR/bundle" --valid "$FACTS_DATA/valid.tsv" --limit 200 --output "$TMPDIR/reproduced-sample"
python3 tools/facts/chat/evaluate.py --binary "$BIN" --facts-binary "$CARGO_TARGET_DIR/release/glassmind-facts" --predictor "$CARGO_TARGET_DIR/release/examples/continuous_predictions" --facts-dir "$TMPDIR/bundle" --valid "$FACTS_DATA/valid.tsv" --output "$TMPDIR/reproduced-full"
for mode in default facts; do
    extra=()
    if [[ "$mode" == facts ]]; then extra+=(--facts-dir "$TMPDIR/bundle"); fi
    python3 tools/facts/chat/legacy.py --binary "$BIN" "${extra[@]}" --output "$TMPDIR/reproduced-$mode"
    python3 tools/facts/chat/legacy.py --binary "$BIN" "${extra[@]}" --model "$INTENT_MODEL" --output "$TMPDIR/reproduced-$mode-model"
done
python3 tools/facts/chat/mixed.py --binary "$BIN" --facts-dir "$TMPDIR/bundle" --output "$TMPDIR/reproduced-mixed"
python3 tools/facts/chat/mixed.py --binary "$BIN" --facts-dir "$TMPDIR/bundle" --model "$INTENT_MODEL" --output "$TMPDIR/reproduced-mixed-model"
