#!/usr/bin/env bash
# Reproduce the rejected CAL result and optional-absent DEV/legacy regression.
# Run only in a clean isolated checkout of the handoff base; no model is installed.
set -euo pipefail
: "${TMPDIR:?persistent scratch required}" "${CARGO_TARGET_DIR:?existing target required}"
export PYTHONDONTWRITEBYTECODE=1
root=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full
run=$TMPDIR/spansequence
mkdir -p "$run"
git diff --quiet -- crates/dialogue
git apply --check tools/dialogue/spansequence/rejected.patch
git apply tools/dialogue/spansequence/rejected.patch
trap 'git apply -R tools/dialogue/spansequence/rejected.patch' EXIT
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt -p glassmind-dialogue -- --check
cap cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
cap cargo test --release -p glassmind-dialogue --lib -j1 -- --test-threads 4
cap cargo build --release -p glassmind-dialogue -j1
cap python tools/dialogue/spansequence/prepare.py --root "$root" --output "$run"
set +e
cap "$CARGO_TARGET_DIR/release/dialogue-spansequence-train" --input "$run/training.json" --output "$run/sequence.json" --report "$run/train-report.json" > "$run/training.log" 2>&1
result=$?
set -e
# Exit 1 is expected: no preregistered threshold passes. Never lower the gate.
test "$result" = 1
python - "$run/train-report.json" <<'PY'
import json,sys
r=json.load(open(sys.argv[1]));assert r['selected'] is None
assert r['cal_proposals_correct']==[[535,512],[533,510],[528,507],[517,499],[505,490],[470,458]]
PY
cap python tools/dialogue/spansequence/cal_audit.py
bash tools/dialogue/spansequence/evaluate.sh
