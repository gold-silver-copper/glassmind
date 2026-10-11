#!/usr/bin/env bash
set -euo pipefail
# Invoke in the repository. RUN must have the approved baseline/ bundle.
: "${RUN:?set RUN to an isolated run directory containing baseline/}"
: "${CARGO_TARGET_DIR:?use the existing assigned target directory}"
: "${SOURCE:?set SOURCE to the pinned EWT TRAIN/DEV directory}"
export TMPDIR="$RUN/tmp"
mkdir -p "$TMPDIR" "$RUN/candidate"
capped_cargo() {
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
capped_cargo fmt -p glassmind-reading -- --check > "$RUN/candidate/fmt.log" 2>&1
capped_cargo test -p glassmind-reading --lib -j1 -- --test-threads 4 > "$RUN/candidate/tests.log" 2>&1
capped_cargo clippy -p glassmind-reading --all-targets -j1 -- -D warnings > "$RUN/candidate/clippy.log" 2>&1
capped_cargo build --release -p glassmind-reading --bin reading-syntax -j1 > "$RUN/candidate/build.log" 2>&1
cp "$CARGO_TARGET_DIR/release/reading-syntax" "$RUN/candidate/reading-syntax"
rlib=("$CARGO_TARGET_DIR"/release/deps/libglassmind_reading-*.rlib)
rustc --edition=2024 tools/reading/syntax/model_check.rs -L "dependency=$CARGO_TARGET_DIR/release/deps" --extern "glassmind_reading=${rlib[0]}" -o "$RUN/candidate/model-check"
"$RUN/candidate/model-check" "$RUN/baseline/model.bin" > "$RUN/candidate/model-check.log"
rustc --edition=2024 tools/reading/syntax-normalized/model-contract.rs -L "dependency=$CARGO_TARGET_DIR/release/deps" --extern "glassmind_reading=${rlib[0]}" -o "$RUN/candidate/beam-model-check"
"$RUN/candidate/beam-model-check" "$RUN/baseline/model.bin" > "$RUN/candidate/beam-model-check.log"
python3 tools/reading/syntax-normalized/evaluate.py --run "$RUN" --source "$SOURCE" > "$RUN/candidate/evaluate.log" 2>&1
