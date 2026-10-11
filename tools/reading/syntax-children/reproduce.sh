#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?on-disk TMPDIR required}"
: "${CARGO_TARGET_DIR:?existing target required}"
SOURCE=${SOURCE:-/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source}
BASELINE=${BASELINE:-/home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run/baseline}
OUT=${OUT:-/home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run/candidate}
mkdir -p "$OUT"
export PYTHONDONTWRITEBYTECODE=1
scoped() {
 TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
 setsid --wait timeout --kill-after=10s 600 "$@"
}
# Runtime patch must be applied if this is replayed after a rejected experiment.
sha256sum -c tools/reading/syntax-children/preregistered.sha256
scoped cargo fmt -p glassmind-reading -- --check
scoped cargo test -p glassmind-reading -j1 -- --test-threads 4
scoped cargo clippy -p glassmind-reading --all-targets -j1 -- -D warnings
scoped cargo build --release -p glassmind-reading --bin reading-syntax -j1
cp "$CARGO_TARGET_DIR/release/reading-syntax" "$OUT/reading-syntax"
# Refuse accidental retraining/overwrite of the original experiment.
test ! -e "$OUT/model.bin"
scoped python3 tools/reading/syntax/measure.py "$OUT/train-time.json" "$OUT/reading-syntax" train \
 --child-features --train "$SOURCE/en_ewt-ud-train.conllu" --model "$OUT/model.bin" --report "$OUT/train-report.json"
mkdir -p "$OUT/source"
cp crates/reading/src/syntax.rs "$OUT/source/syntax.rs"
cp crates/reading/src/bin/reading-syntax.rs "$OUT/source/reading-syntax.rs"
git diff -- crates/reading/src/syntax.rs crates/reading/src/bin/reading-syntax.rs > "$OUT/runtime.patch"
sha256sum crates/reading/src/syntax.rs crates/reading/src/bin/reading-syntax.rs \
 "$OUT/model.bin" "$OUT/reading-syntax" tools/reading/syntax-children/{PLAN.md,authored.jsonl,compare.py,probes.py,model_metadata.py,reproduce.sh} \
 tools/reading/syntax/{evaluate.py,contracts.py,audit_train.py,measure.py,model_check.rs,authored.jsonl} \
 tools/reading/raw-syntax/authored.jsonl tools/reading/syntax/parent/audit.py \
 "$SOURCE/en_ewt-ud-train.conllu" "$SOURCE/en_ewt-ud-dev.conllu" > "$OUT/freeze.sha256"
python3 tools/reading/syntax-children/model_metadata.py "$BASELINE" "$OUT"
for mode in sample full; do
 python3 tools/reading/syntax/evaluate.py --source "$SOURCE" --artifacts "$OUT" \
 --binary "$OUT/reading-syntax" --model "$OUT/model.bin" --mode "$mode"
done
python3 tools/reading/syntax/parent/audit.py --source "$SOURCE" --predictions "$OUT/full-predictions.jsonl" --out "$OUT/audit"
python3 tools/reading/syntax-children/compare.py "$SOURCE" "$BASELINE" "$OUT"
scoped python3 tools/reading/syntax-children/probes.py "$BASELINE" "$OUT"
scoped python3 tools/reading/syntax/contracts.py "$OUT"
scoped python3 tools/reading/syntax/audit_train.py "$SOURCE" "$OUT"
# Replay legacy weights with the new binary; compare against supplied old outputs.
for kind in forms raw; do
 input="$BASELINE/forms.jsonl"
 if [ "$kind" = raw ]; then input="$BASELINE/raw-inputs.jsonl"; fi
 scoped "$OUT/reading-syntax" predict --model "$BASELINE/model.bin" --input "$input" --output "$OUT/legacy-$kind.jsonl"
 cmp "$BASELINE/$kind-predictions.jsonl" "$OUT/legacy-$kind.jsonl"
done
TASK_READING_RLIB=$(rg --files "$CARGO_TARGET_DIR/release/deps" -g 'libglassmind_reading-*.rlib' | sort | head -1)
scoped rustc --edition=2024 tools/reading/syntax/model_check.rs \
 -L "dependency=$CARGO_TARGET_DIR/release/deps" --extern "glassmind_reading=$TASK_READING_RLIB" -o "$OUT/model-check"
scoped "$OUT/model-check" "$BASELINE/model.bin"
scoped "$OUT/model-check" "$OUT/model.bin"
mkdir -p "$OUT/legacy-contracts"
ln -s "$OUT/reading-syntax" "$OUT/legacy-contracts/reading-syntax"
ln -s "$BASELINE/model.bin" "$OUT/legacy-contracts/model.bin"
scoped python3 tools/reading/syntax/contracts.py "$OUT/legacy-contracts"
sha256sum -c "$OUT/freeze.sha256"
