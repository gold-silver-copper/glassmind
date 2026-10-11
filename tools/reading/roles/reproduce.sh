#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?Set disk-backed TMPDIR, never /tmp}"
: "${CARGO_TARGET_DIR:?Use the existing target directory}"
: "${ROLES_RUN:?Set a fresh artifact directory outside git}"
: "${ROLES_BASE:?Set the frozen baseline directory}"
: "${SQUAD_SOURCE:?Set readonly existing SQuAD source directory}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
mkdir -p "$ROLES_RUN"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap python3 tools/reading/roles/experiment.py prepare
cap cargo fmt -p glassmind-reading -- --check
cap cargo build --release -p glassmind-reading --bin reading-qa -j1
cap cargo test -p glassmind-reading --lib -j1 qa::tests -- --test-threads 4
cap cargo test -p glassmind-reading --bin reading-probability -j1 probability::tests -- --test-threads 4
cp "$CARGO_TARGET_DIR/release/reading-qa" "$ROLES_RUN/reading-qa"
cap "$ROLES_RUN/reading-qa" train --feature-version 2 --input "$ROLES_BASE/fit.train.jsonl" --model "$ROLES_RUN/model.json"
for stage in freeze sample full probes; do
 cap python3 tools/reading/roles/experiment.py "$stage" --binary "$ROLES_RUN/reading-qa"
done
