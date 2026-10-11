#!/bin/bash
set -euo pipefail
work=/home/gsc/.cache/glassmind/continuous-20261010
run="$work/coordinator/ambiguity-check"
worker="$work/repairambiguity2-run"
export TMPDIR="$run/tmp"
export CARGO_TARGET_DIR="$run/target"
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 900 "$@"; }
cap python3 tools/coding-repair-dev.py --binary "$run/glassmind" --train "$worker/dev-inputs/train" --sources "$worker/dev-inputs/sources" --sample --out "$run/sample"
cap python3 tools/coding-repair-dev.py --binary "$run/glassmind" --train "$worker/dev-inputs/train" --sources "$worker/dev-inputs/sources" --out "$run/full" --check-cache "$run/sample/checks"
witness=$(rg --files "$run/full/scratch" -g witness.json)
test -n "$witness"
cap python3 tools/coding-ambiguity/replay_witness.py --witness "$witness" --root "$worker/dev-inputs/sources/rb-fix-tests-leap-year" --out "$run/native-witness"
