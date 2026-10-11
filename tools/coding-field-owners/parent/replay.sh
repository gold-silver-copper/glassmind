#!/usr/bin/env bash
set -euo pipefail
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/fieldowners-review/tmp
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/fieldowners-review/scratch
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run/target
export CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true
root=/home/gsc/.cache/glassmind/continuous-20261010
scripts=$root/fieldowners-run/submitted/tools/coding-field-owners
out=$root/coordinator/fieldowners-review
python3 "$scripts/evaluate.py" --binary "$root/fieldowners-run/glassmind-accepted" --output "$out/worker-sample" --sample
python3 "$scripts/evaluate.py" --binary "$root/fieldowners-run/glassmind-accepted" --output "$out/worker-full"
python3 "$scripts/audit.py" --binary "$root/fieldowners-run/glassmind-accepted" --output "$out/worker-audit"
