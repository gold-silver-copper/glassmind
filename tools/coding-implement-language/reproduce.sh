#!/usr/bin/env bash
# Replays retained candidate binaries, not the restored production runtime.
set -euo pipefail
RUN=/home/gsc/.cache/glassmind/continuous-20261010/impllanguage-run
: "${CARGO_TARGET_DIR:?Reuse the existing target directory}"
: "${TMPDIR:?Set an on-disk TMPDIR outside /tmp}"
OUT=${1:?Supply a new absolute output directory}
BIN="$RUN/glassmind-final"
sha256sum -c tools/coding-implement-language/FROZEN.sha256
python3 - "$BIN" <<'PY'
import hashlib,sys
assert hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest()=='ef5ba7d56a22814f620c322b0205e6a13234c7903aa7325c1eabc1c8096b1f5d'
PY
# Outer scope also caps inherited fallback commands in negative CLI cases.
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/coding-implement-language/adapter.py --binary "$BIN" --mode sample --no-env --out "$OUT/sample"
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/coding-implement-language/adapter.py --binary "$BIN" --mode train --out "$OUT/train"
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/coding-implement-language/evaluate.py --binary "$BIN" --sample --out "$OUT/original-sample"
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/coding-implement-language/evaluate.py --binary "$BIN" --out "$OUT/original-full"
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/coding-implement-language/environment.py --binary "$BIN" --out "$OUT/environment"
python3 tools/coding-implement-language/coverage.py "$RUN/parser-probe-final" > "$OUT/parser-coverage.json"
# Expected exit1: repair x invokes unchanged repair behavior and creates artifacts.
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/coding-implement-language/adapter.py --binary "$BIN" --mode contracts --no-env --out "$OUT/contracts"
