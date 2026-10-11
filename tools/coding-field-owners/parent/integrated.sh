#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/fieldowners-review/tmp
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/fieldowners-review/scratch
export CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true
out=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/fieldowners-review
bash tools/check.sh > "$out/workspace.log" 2>&1
cargo test -p glassmind-world-rust -j1 -- --test-threads 4 > "$out/world-tests.log" 2>&1
cp target/release/glassmind "$out/glassmind-integrated"
sha256sum "$out/glassmind-integrated"
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/fieldowners-run/target
python3 tools/coding-field-owners/evaluate.py --binary "$out/glassmind-integrated" --output "$out/integrated-sample" --sample
python3 tools/coding-field-owners/evaluate.py --binary "$out/glassmind-integrated" --output "$out/integrated-full"
python3 tools/coding-field-owners/audit.py --binary "$out/glassmind-integrated" --output "$out/integrated-audit"
