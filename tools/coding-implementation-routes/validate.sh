#!/usr/bin/env bash
set -u
RUN=/home/gsc/.cache/glassmind/continuous-20261010/implementationroutes-run
TOOL=tools/coding-implementation-routes
BIN="$RUN/glassmind-candidate"
cp "$CARGO_TARGET_DIR/debug/glassmind" "$BIN"
rustc --edition=2024 "$TOOL/parser_probe.rs" -L "dependency=$CARGO_TARGET_DIR/debug/deps" --extern "glassmind_english_rust=$(find "$CARGO_TARGET_DIR/debug/deps" -maxdepth 1 -name 'libglassmind_english_rust-*.rlib' -print -quit)" -o "$RUN/parser-probe"
run() {
 local tag=$1
 shift
 systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@" > "$TOOL/reports/$tag.log" 2>&1
 echo "$tag $?" | tee -a "$TOOL/reports/status.log"
}
run baseline-sample python3 "$TOOL/adapter.py" --binary "$RUN/baseline/glassmind" --mode sample --no-env --out "$RUN/baseline-sample"
run sample python3 "$TOOL/adapter.py" --binary "$BIN" --mode sample --no-env --out "$RUN/sample"
run authored-sample python3 "$TOOL/evaluate.py" --binary "$BIN" --sample --out "$RUN/authored-sample"
run train python3 "$TOOL/adapter.py" --binary "$BIN" --mode train --out "$RUN/train"
run authored-full python3 "$TOOL/evaluate.py" --binary "$BIN" --out "$RUN/authored-full"
run contracts python3 "$TOOL/adapter.py" --binary "$BIN" --mode contracts --no-env --out "$RUN/contracts"
run environment python3 "$TOOL/environment.py" --binary "$BIN" --out "$RUN/environment"
run boundaries python3 "$TOOL/boundaries.py" --binary "$BIN" --probe "$RUN/parser-probe" --out "$RUN/boundaries"
run isolated python3 "$TOOL/isolated.py" "$BIN" "$RUN/isolated"
python3 "$TOOL/coverage.py" "$RUN/parser-probe" > "$TOOL/reports/parser-coverage.json"
