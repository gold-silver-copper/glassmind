#!/usr/bin/env bash
set -u
binary=$1
out=$2
mkdir -p "$out"
for adapter in coding-metatypes-dev coding-answerfiles-dev coding-answerfiles-regression coding-addfields-dev coding-edits-dev; do
  extra=()
  if [[ "$adapter" == coding-addfields-dev ]]; then extra=(--independent); fi
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python "tools/$adapter.py" --binary "$binary" --output "$out/$adapter" "${extra[@]}" > "$out/$adapter.log" 2>&1
  printf '%s %s\n' "$adapter" "$?" >> "$out/status.txt"
done
