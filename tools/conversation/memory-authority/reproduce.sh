#!/usr/bin/env bash
# Run from repository root. No fetch, training, or held-out evaluation.
set -euo pipefail
: "${TMPDIR:?Set an on-disk TMPDIR outside /tmp}"
: "${CARGO_TARGET_DIR:?Use the existing target directory}"
: "${BASELINE:?Path to production 9b875b8 glassmind binary}"
: "${MODEL:?Path to production everyday intent.json}"
OUT=${OUT:-tools/conversation/memory-authority/replay}
mkdir -p "$OUT"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
sha256sum -c tools/conversation/memory-authority/frozen.sha256
cap cargo fmt --all -- --check
cap cargo build -p glassmind -p glassmind-everyday --release -j1
cap cargo test -p glassmind-everyday --release --lib -j1 -- --test-threads 4
cap cargo test -p glassmind --release everyday::tests -j1 -- --test-threads 4
cap cargo clippy -p glassmind-everyday -p glassmind --all-targets -j1 -- -D warnings
CANDIDATE="$CARGO_TARGET_DIR/release/glassmind"
for version in baseline candidate; do
  binary="$BASELINE"
  if [[ "$version" == candidate ]]; then binary="$CANDIDATE"; fi
  for mode in plain model; do
    extra=()
    if [[ "$mode" == model ]]; then extra=(--model "$MODEL"); fi
    python3 tools/conversation/memory-authority/evaluate.py "$binary" "$OUT/$version-sample-$mode" --sample "${extra[@]}"
    python3 tools/conversation/memory-authority/evaluate.py "$binary" "$OUT/$version-full-$mode" "${extra[@]}"
  done
  python3 tools/talk-dev.py "$binary" --model "$MODEL" --output "$OUT/$version-calc27.json"
  python3 tools/talk-story-dev.py "$binary" --model "$MODEL" --output "$OUT/$version-story47.json"
done
python3 - "$OUT" <<'PY'
import json,sys
from pathlib import Path
root=Path(sys.argv[1])
for mode in ['plain','model']:
 r=json.loads((root/f'candidate-full-{mode}/report.json').read_text())
 assert r['contracts']=={'right':93,'total':93},r['contracts']
 assert r['original']=={'right':41,'total':41},r['original']
for suite in ['calc27','story47']:
 b=json.loads((root/f'baseline-{suite}.json').read_text())
 c=json.loads((root/f'candidate-{suite}.json').read_text())
 # Provenance necessarily names a different executable, never remove answers.
 b.pop('binary_sha256',None);c.pop('binary_sha256',None)
 assert b==c,suite
PY
python3 tools/facts/sets/contracts.py --binary "$CANDIDATE" --output "$OUT/facts-contracts"
python3 tools/facts/sets/session_contracts.py --binary "$CANDIDATE" --bundle "$OUT/facts-contracts/bundle" --output "$OUT/facts-session"
python3 tools/conversation/memory-authority/facts_equivalence.py "$BASELINE" "$CANDIDATE" "$MODEL" "$OUT/facts-contracts/bundle" "$OUT/facts-equivalence"
# Supplemental review checks deliberately retain inherited talk failures.
sha256sum -c tools/conversation/memory-authority/learned-social-frozen.sha256
for version in baseline candidate; do
  binary="$BASELINE"
  if [[ "$version" == candidate ]]; then binary="$CANDIDATE"; fi
  python3 tools/conversation/memory-authority/learned_social.py "$binary" "$OUT/$version-social"
  python3 tools/conversation/memory-authority/learned_social.py "$binary" "$OUT/$version-social-model" --model "$MODEL"
done
EVERYDAY_BINARY="$CARGO_TARGET_DIR/release/glassmind-everyday" python3 tools/conversation/memory-authority/learned_social.py tools/conversation/memory-authority/standalone-chat.sh "$OUT/candidate-social-standalone"
EVERYDAY_BINARY="$CARGO_TARGET_DIR/release/glassmind-everyday" python3 tools/conversation/memory-authority/learned_social.py tools/conversation/memory-authority/standalone-chat.sh "$OUT/candidate-social-standalone-model" --model "$MODEL"
