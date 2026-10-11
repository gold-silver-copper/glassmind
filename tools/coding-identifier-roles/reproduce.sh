#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?Supply persistent disk TMPDIR outside /tmp}"
: "${CARGO_TARGET_DIR:?Use existing target directory}"
[[ "$TMPDIR" != /tmp* ]]
export CARGO_NET_OFFLINE=true
run=${TMPDIR%/tmp}
out=${1:-$run/identifier-replay}
baseline=${2:-$run/baseline/glassmind}
mkdir -p "$out"
export GLASSMIND_SCRATCH="$out/scratch"
scope=(systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600)
sha256sum -c tools/coding-identifier-roles/FROZEN.sha256
"${scope[@]}" cargo fmt -p glassmind-english -p glassmind-english-rust -- --check
"${scope[@]}" cargo test -p glassmind-english -p glassmind-english-rust -j1 -- --test-threads 4
"${scope[@]}" cargo clippy -p glassmind-english -p glassmind-english-rust -p glassmind --all-targets -j1 -- -D warnings
"${scope[@]}" cargo build -p glassmind -j1
cp "$CARGO_TARGET_DIR/debug/glassmind" "$out/glassmind"
for mode in baseline candidate; do
  binary=$baseline
  if [[ "$mode" == candidate ]]; then binary="$out/glassmind"; fi
  python tools/coding-identifier-roles/evaluate.py --binary "$binary" --output "$out/$mode-sample" --sample
  python tools/coding-identifier-roles/evaluate.py --binary "$binary" --output "$out/$mode-full"
  bash tools/coding-identifier-roles/regressions.sh "$binary" "$out/regressions-$mode"
  bash tools/coding-identifier-roles/addfield-cases.sh "$binary" "$out/addfield-$mode"
done
python - "$out" <<'PY'
import json,sys
from pathlib import Path
root=Path(sys.argv[1]);b=json.loads((root/'baseline-full/results.json').read_text());c=json.loads((root/'candidate-full/results.json').read_text())
assert len(b)==len(c)==42
assert all(r['pass_'] for r in c)
assert all(not x['pass_'] or y['pass_'] for x,y in zip(b,c))
print('Frozen gate:',sum(r['pass_'] for r in b),'->',sum(r['pass_'] for r in c),'of',len(c))
assert (root/'addfield-baseline/status.txt').read_text()==(root/'addfield-candidate/status.txt').read_text()
print('Original regression statuses are retained, including known export and live-rollback failures.')
PY
