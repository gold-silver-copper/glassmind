#!/bin/bash
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
unset OPENAI_API_KEY ANTHROPIC_API_KEY GOOGLE_API_KEY GEMINI_API_KEY
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/timedmatch-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/timedmatch-run/target
exec systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"
