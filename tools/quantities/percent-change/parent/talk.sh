#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
out=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/percentchange-review
export TMPDIR="$out/tmp" GLASSMIND_SCRATCH="$out/scratch"
bash "$out/talk-regressions.sh"
for mode in plain model; do
 args=(); if [[ "$mode" == model ]]; then args=(--model /home/gsc/.cache/glassmind/models/everyday/intent.json); fi
 python3 tools/conversation/social-routing/evaluate.py "$out/glassmind-integrated" "$out/social-sample-$mode" --sample "${args[@]}"
 python3 tools/conversation/social-routing/evaluate.py "$out/glassmind-integrated" "$out/social-full-$mode" "${args[@]}"
 python3 tools/conversation/memory-authority/evaluate.py "$out/glassmind-integrated" "$out/memory-$mode" "${args[@]}"
done
python3 "$out/talk-probes.py"
