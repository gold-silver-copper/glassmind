#!/usr/bin/env bash
# Replays the archived rejected candidate. Does not build or run a benchmark.
set -euo pipefail
: "${TMPDIR:?persistent disk TMPDIR required}"
: "${CARGO_TARGET_DIR:?reuse existing target}"
RUN=/home/gsc/.cache/glassmind/continuous-20261010/implementationroutes-run
TOOL=tools/coding-implementation-routes
OUT=${1:?new absolute output directory required}
BIN="$RUN/glassmind-candidate"
sha256sum -c "$TOOL/FROZEN.sha256"
python3 - "$BIN" <<'PY'
import hashlib,sys
assert hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest()=='1966eb5735d79540432335ac67438cc79336c3d370155764ecc878bc322ca4ca'
PY
run() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
run python3 "$TOOL/adapter.py" --binary "$BIN" --mode sample --no-env --out "$OUT/sample"
run python3 "$TOOL/adapter.py" --binary "$BIN" --mode train --out "$OUT/train"
run python3 "$TOOL/evaluate.py" --binary "$BIN" --sample --out "$OUT/authored-sample"
run python3 "$TOOL/evaluate.py" --binary "$BIN" --out "$OUT/authored-full"
run python3 "$TOOL/adapter.py" --binary "$BIN" --mode contracts --no-env --out "$OUT/contracts"
run python3 "$TOOL/environment.py" --binary "$BIN" --out "$OUT/environment"
run python3 "$TOOL/isolated_calculate.py" "$BIN" "$OUT/isolated-calculate"
python3 "$TOOL/coverage.py" "$RUN/parser-probe" > "$OUT/parser.json"
# Frozen gate fails36/46; preserve the failure rather than weaken it.
run python3 "$TOOL/boundaries.py" --binary "$BIN" --probe "$RUN/parser-probe" --out "$OUT/boundaries"
