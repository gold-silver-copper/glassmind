#!/usr/bin/env bash
set -euo pipefail
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/contextledger-run/tmp
: "${CARGO_TARGET_DIR:?existing target required}"
D=tools/dialogue/context-ledger
run() {
 printf '%q ' "$@" >> "$D/commands.log"; printf '\n' >> "$D/commands.log"
 "$@"
}
cap() { run systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt --all
cap cargo test -p glassmind-dialogue --example context-ledger -j1 -- --test-threads 4
cap cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
cap cargo build -p glassmind-dialogue --release --example context-ledger -j1
cp "$CARGO_TARGET_DIR/release/examples/context-ledger" "$D/artifacts/context-ledger"
cap "$D/artifacts/context-ledger" "$D/artifacts/dataset.json" "$D/artifacts" /home/gsc/.cache/glassmind/continuous-20261010/contextledger-source/rejected-model.json
