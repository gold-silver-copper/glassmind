#!/usr/bin/env bash
set -u
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}" "${GLASSMIND_SCRATCH:?}"
binary=$1
out=$2
mkdir -p "$out"
for adapter in coding-metatypes-dev coding-answerfiles-dev coding-answerfiles-regression coding-edits-dev coding-variants-dev; do
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python "tools/$adapter.py" --binary "$binary" --output "$out/$adapter" > "$out/$adapter.log" 2>&1
  printf '%s %s\n' "$adapter" "$?" >> "$out/status.txt"
done
bash tools/coding-identifier-roles/addfield-cases.sh "$binary" "$out/addfield"
