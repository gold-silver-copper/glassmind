#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${CARGO_TARGET_DIR:?Use the provided existing target directory}"
: "${TMPDIR:?Use an on-disk worker TMPDIR}"
case "$TMPDIR" in /tmp|/tmp/*) exit 2;; esac
baseline=$(realpath "${1:?Path to frozen baseline binary}")
artifacts=$(realpath -m "${2:?New artifact output directory}")
mkdir "$artifacts"
export GLASSMIND_SCRATCH="$artifacts/scratch"
cargo_step() {
    TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
python3 tools/coding-askverify/run.py --binary "$baseline" --out "$artifacts/baseline-sample" --sample
python3 tools/coding-askverify/run.py --binary "$baseline" --out "$artifacts/baseline-full"
python3 tools/coding-askverify/supplement.py --binary "$baseline" --out "$artifacts/baseline-supplement"
cargo_step fmt -p glassmind --check > "$artifacts/fmt.log" 2>&1
cargo_step test -p glassmind -j1 -- --test-threads 4 > "$artifacts/tests.log" 2>&1
cargo_step clippy -p glassmind --all-targets -j1 -- -D warnings > "$artifacts/clippy.log" 2>&1
cargo_step build -p glassmind -j1 > "$artifacts/build.log" 2>&1
cp "$CARGO_TARGET_DIR/debug/glassmind" "$artifacts/glassmind"
python3 tools/coding-askverify/run.py --binary "$artifacts/glassmind" --out "$artifacts/candidate-sample" --sample
python3 tools/coding-askverify/run.py --binary "$artifacts/glassmind" --out "$artifacts/candidate-full" --compare "$artifacts/baseline-full/results.json"
python3 tools/coding-askverify/supplement.py --binary "$artifacts/glassmind" --out "$artifacts/candidate-supplement"
cargo_step test --manifest-path "$artifacts/candidate-full/field/package/Cargo.toml" -j1 -- --test-threads 4 > "$artifacts/original-fixture-test.log" 2>&1
sha256sum "$baseline" "$artifacts/glassmind" > "$artifacts/binary-hashes.txt"
python3 - "$artifacts" <<'PY'
import json, sys
from pathlib import Path
root = Path(sys.argv[1])
for name in ('candidate-sample', 'candidate-full', 'candidate-supplement'):
    report = json.loads((root / name / 'results.json').read_text())
    assert report['passed'] == report['total'], name
    assert all(report.get('byte_compatibility', {}).values()), name
PY
