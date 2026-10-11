#!/usr/bin/env bash
set -euo pipefail
cd /home/gsc/code/glassmind
root=/home/gsc/.cache/glassmind/continuous-20261010
out=$root/coordinator/percentchange-review
export TMPDIR="$out/tmp" GLASSMIND_SCRATCH="$out/scratch" CARGO_TARGET_DIR=/home/gsc/code/glassmind/target
export CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true
export PYTHONPATH="$root/quantities-run/tmp/quantities-deps"
bash tools/check.sh > "$out/workspace.log" 2>&1
cargo test -p glassmind-quantities -j1 -- --test-threads 4 > "$out/tests.log" 2>&1
cp target/release/glassmind "$out/glassmind-integrated"
cp target/release/glassmind-quantities "$out/glassmind-quantities"
sha256sum "$out/glassmind-integrated" "$out/glassmind-quantities"
python3 "$out/independent-integrated.py"
python3 tools/quantities/percent-change/probes.py "$root/percentchange-run/baseline/quantities" "$out/glassmind-quantities" "$out/integrated/probes.json"
cp "$root/percentchange-run/baseline/quantities" "$TMPDIR/glassmind-quantities-baseline"
export CANDIDATE_BINARY="$out/glassmind-quantities"
python3 tools/quantities/linearwords/check.py > "$out/integrated/regressions.log"
cp "$TMPDIR/checks.json" "$out/integrated/regressions.json"
python3 tools/quantities/linearwords/absolute.py --binary "$out/glassmind-quantities" --output "$out/integrated/absolute.json"
python3 tools/quantities/percent-change/evaluate.py --binary "$out/glassmind-quantities" --data "$root/quantities-run/tmp/mathqa" --output "$out/integrated/sample" --sample
python3 tools/quantities/percent-change/evaluate.py --binary "$out/glassmind-quantities" --data "$root/quantities-run/tmp/mathqa" --output "$out/integrated/full"
