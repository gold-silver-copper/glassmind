#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?persistent TMPDIR required}" "${CARGO_TARGET_DIR:?existing target required}"
: "${PROPOSAL_RUN:?new external frozen-candidate directory required}"
export PROPOSAL_SOURCE=${PROPOSAL_SOURCE:-/home/gsc/.cache/glassmind/continuous-20261010/equation-source}
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
export TMPDIR CARGO_TARGET_DIR
if [[ ! -f crates/quantities/src/proposal.rs ]]; then
  echo 'Rejected runtime is intentionally absent. In an isolated checkout, first apply tools/quantities/equation-proposals/rejected-runtime.patch.' >&2
  exit 2
fi
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo build -p glassmind-quantities --release --examples --bins -j1
cap cargo test -p glassmind-quantities -j1 -- --test-threads 4
cap python3 tools/quantities/equation-proposals/experiment.py
cap python3 tools/quantities/equation-proposals/contracts.py
