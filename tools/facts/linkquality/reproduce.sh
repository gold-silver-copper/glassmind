#!/usr/bin/env bash
# Run from repo root, after applying rejected.patch in a disposable checkout.
# This is a rejected experiment. Do not deploy it.
set -euo pipefail
RUN=${RUN:-/home/gsc/.cache/glassmind/continuous-20261010/linkquality-run}
DATA=${DATA:-/home/gsc/.cache/glassmind/data/simplequestions}
export TMPDIR="$RUN/tmp"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$RUN/target}
OUT=${OUT:-$RUN/replay}
mkdir -p "$OUT" "$TMPDIR"
cap() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
cap cargo fmt -p glassmind-facts --check
cap cargo test -p glassmind-facts -j1 -- --test-threads 4
cap cargo clippy -p glassmind-facts --all-targets -j1 -- -D warnings
cap cargo build --release -p glassmind-facts --example linkquality_predictions -j1
for split in train valid; do
  cut -f4 "$DATA/$split.tsv" > "$OUT/$split.questions"
done
for scope in old500 new999; do
  index=tools/facts/index.json
  if [ "$scope" = new999 ]; then index="$RUN/baseline/train-index.json"; fi
  for limit in 200 2821; do
    "$RUN/baseline/glassmind-facts" score --input "$DATA/valid.tsv" \
      --model "$DATA/relation.json" --index "$index" --limit "$limit" \
      > "$OUT/$scope-base-$limit.json"
  done
  for split in train valid; do
    cap "$RUN/baseline/predictions" "$DATA/relation.json" "$index" \
      < "$OUT/$split.questions" > "$OUT/$scope-$split-frozen.jsonl"
  done
  for split in train sample valid; do
    input="$OUT/$split.questions"
    if [ "$split" = sample ]; then head -n200 "$OUT/valid.questions" > "$input"; fi
    cap "$CARGO_TARGET_DIR/release/examples/linkquality_predictions" \
      "$DATA/relation.json" "$index" '"interrogative_determiner_v1"' \
      < "$input" > "$OUT/$scope-$split-pairs.jsonl"
    extra=(--frozen "$OUT/$scope-$split-frozen.jsonl")
    scored_split=$split
    if [ "$split" = sample ]; then scored_split=valid; extra=(--limit 200); fi
    python3 tools/facts/linkquality/audit.py --data "$DATA" --split "$scored_split" \
      --pairs "$OUT/$scope-$split-pairs.jsonl" --rows-out "$OUT/$scope-$split-audit.jsonl" \
      "${extra[@]}" > "$OUT/$scope-$split-summary.json"
  done
done
# Optional original-artifact overlap replay (requires $RUN/artifacts retained):
python3 tools/facts/linkquality/overlap.py "$RUN" "$DATA" > "$OUT/overlap.json"
