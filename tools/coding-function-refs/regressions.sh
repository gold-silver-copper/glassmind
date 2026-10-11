#!/usr/bin/env bash
set -u
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
binary=$(realpath "$1")
out=$(realpath -m "$2")
mkdir -p "$out"
export GLASSMIND_SCRATCH="$out/scratch" CARGO_NET_OFFLINE=true
for adapter in coding-edits-dev coding-variants-dev coding-packages-dev coding-metatypes-dev coding-answerfiles-dev coding-answerfiles-regression; do
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python "tools/$adapter.py" --binary "$binary" --output "$out/$adapter" > "$out/$adapter.log" 2>&1
  printf '%s %s\n' "$adapter" "$?" >> "$out/status.txt"
done
python tools/coding-field-owners/evaluate.py --binary "$binary" --output "$out/fields" > "$out/fields.log" 2>&1
printf 'fields %s\n' "$?" >> "$out/status.txt"
python tools/coding-askverify/run.py --binary "$binary" --out "$out/askverify" > "$out/askverify.log" 2>&1
printf 'askverify %s\n' "$?" >> "$out/status.txt"
bash tools/coding-identifier-roles/addfield-cases.sh "$binary" "$out/addfield" > "$out/addfield.log" 2>&1
printf 'addfield %s\n' "$?" >> "$out/status.txt"
