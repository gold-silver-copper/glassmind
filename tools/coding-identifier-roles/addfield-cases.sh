#!/usr/bin/env bash
set -u
binary=$1
out=$2
mkdir -p "$out"
for name in alias-self unrelated default collision alias unknown-macro update cfg trailing unsafe-default live-rollback unsafe-function test-fails type-mismatch local-shadow; do
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python tools/coding-addfields-dev.py --binary "$binary" --output "$out/$name" --independent --cases "$name" > "$out/$name.log" 2>&1
  printf '%s %s\n' "$name" "$?" >> "$out/status.txt"
done
