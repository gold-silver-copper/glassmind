# Actual commands and reproduction

Working directory: `/home/gsc/.cache/glassmind/continuous-20261010/udsyntax`.
Inherited target `/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/target`;
TMPDIR `/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/tmp`.
`OUT=/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/artifacts` and
`SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source` below.
Outer `glassmind-continuous-udsyntax.scope` verified MemoryMax8589934592,
MemorySwapMax0. Every Cargo call used the following prefix, including fmt:

```bash
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 COMMAND
```

Commands executed individually (all logs retained in OUT):

```bash
cargo fmt --all -- --check                 # initial formatting difference, preserved
cargo test -p glassmind-reading --lib -j1 -- --test-threads 4
cargo fmt --all
cargo build --release -p glassmind-reading --bin reading-syntax -j1
cargo clippy --workspace --all-targets -j1 -- -D warnings
# Initial training-launch command below failed127 before executing the binary:
/usr/bin/time -v -o "$OUT/train-time.txt" "$CARGO_TARGET_DIR/release/reading-syntax" train \
  --train "$SOURCE/en_ewt-ud-train.conllu" --model "$OUT/model.bin" --report "$OUT/train-report.json"
# The only actual training run, same scope prefix:
python3 tools/reading/syntax/measure.py "$OUT/train-time.txt" "$CARGO_TARGET_DIR/release/reading-syntax" train \
  --train "$SOURCE/en_ewt-ud-train.conllu" --model "$OUT/model.bin" --report "$OUT/train-report.json"
cargo fmt --all -- --check
cargo test -p glassmind-reading -j1 -- --test-threads 4
cargo build --release --workspace -j1
```

TRAIN audit ran before DEV (after training):
`PYTHONDONTWRITEBYTECODE=1 python3 tools/reading/syntax/audit_train.py "$SOURCE" "$OUT"`.
The initial audit helper double-counted the nonprojective summary through two
counter paths; original `train-audit-initial-error.json` retained. Corrected
before DEV/freeze to287, agreeing with independent preimplementation inspection
and TRAIN oracle exclusion. No model, criterion or scoring change resulted.

A copy of the initial release binary was frozen at OUT/reading-syntax with
model and source hashes in FREEZE.json before first DEV access. Evaluation:

```bash
python3 tools/reading/syntax/evaluate.py --source "$SOURCE" --artifacts "$OUT" \
  --binary "$OUT/reading-syntax" --model "$OUT/model.bin" --mode sample
python3 tools/reading/syntax/evaluate.py --source "$SOURCE" --artifacts "$OUT" \
  --binary "$OUT/reading-syntax" --model "$OUT/model.bin" --mode full
python3 tools/reading/syntax/evaluate.py --source "$SOURCE" --artifacts "$OUT" \
  --binary "$OUT/reading-syntax" --model "$OUT/model.bin" --mode authored
```

The evaluator itself invokes each foreground native prediction under the same
scope/timeout plus `python3 tools/reading/syntax/measure.py TIMING_FILE BINARY ...`.
This reports actual child CPU, wall time and maximum RSS; GNU time is unavailable.
Gold CoNLL-U stays in the offline scorer; only forms go to native inference.

Post-freeze contracts, each native command with the scope prefix above:

```bash
python3 tools/reading/syntax/contracts.py "$OUT"
TASK_READING_RLIB=$(rg --files "$CARGO_TARGET_DIR/release/deps" -g 'libglassmind_reading-*.rlib' | sort | head -1)
rustc --edition=2024 tools/reading/syntax/model_check.rs \
  -L "dependency=$CARGO_TARGET_DIR/release/deps" --extern "glassmind_reading=$TASK_READING_RLIB" \
  -o "$OUT/model-check"
"$OUT/model-check" "$OUT/model.bin"
python3 tools/reading/syntax/measure.py "$OUT/workspace-replay-time.json" \
  "$CARGO_TARGET_DIR/release/reading-syntax" predict --model "$OUT/model.bin" \
  --input "$OUT/full-input.jsonl" --output "$OUT/workspace-predictions.jsonl"
cmp "$OUT/full-predictions.jsonl" "$OUT/workspace-predictions.jsonl"
```

Workspace feature unification changes executable bytes; its complete DEV output
is byte-identical to the frozen candidate. Both binaries retained. No second
training run or hyperparameter search was done. `reproduce.sh` consolidates these
successful operations for a future separate output directory; the consolidated
script was syntax-checked, not rerun as a redundant second experiment.
