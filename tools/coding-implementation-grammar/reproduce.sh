#!/usr/bin/env bash
set -euo pipefail
RUN=/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run
TOOL=tools/coding-implementation-grammar
TAG=${1:?Supply a fresh replay tag}
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
export GLASSMIND_SCRATCH="$RUN/$TAG-scratch"
BIN="$RUN/frozen-candidate/glassmind"
run() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"
}
(cd "$TOOL" && sha256sum -c FROZEN.sha256)
run python3 "$TOOL/run.py" sample "$BIN" "$TAG-sample"
run python3 "$TOOL/run.py" boundaries "$BIN" "$TAG-boundaries"
run python3 "$TOOL/run.py" native "$BIN" "$TAG-native"
run python3 "$TOOL/verify.py" "$TAG-native"
run python3 "$TOOL/environment.py" "$BIN" "$TAG-environment"
run python3 "$TOOL/run.py" train-sample "$BIN" "$TAG-train-sample"
run python3 "$TOOL/verify.py" "$TAG-train-sample"
# One full TRAIN invocation per fresh replay; no DEV or full765.
run python3 "$TOOL/run.py" train "$BIN" "$TAG-train-full"
run python3 "$TOOL/verify.py" "$TAG-train-full"

# Reproduce the rejected acceptance expansion without altering original contracts.
run python3 "$TOOL/audit_freeze.py" "$TAG-freeze-audit"
exit 1
