#!/usr/bin/env bash
# From repository root; no fetches/training/credentials, bounded supplied scratch.
set -euo pipefail
: "${TMPDIR:?Use persistent disk scratch}" "${CARGO_TARGET_DIR:?Use existing target}"
BASELINE=${BASELINE:-../socialrouting-run/baseline/glassmind}
MODEL=/home/gsc/.cache/glassmind/models/everyday/intent.json
OUT=tools/conversation/social-routing/reports
sha256sum -c tools/conversation/social-routing/frozen.sha256
bash tools/conversation/social-routing/validate.sh
BINARY=$CARGO_TARGET_DIR/release/glassmind
for VERSION in baseline candidate; do
 BIN=$BASELINE
 if [[ "$VERSION" == candidate ]]; then BIN=$BINARY; fi
 for MODE in plain model; do
  ARGS=(); SUFFIX=''
  if [[ "$MODE" == model ]]; then ARGS=(--model "$MODEL"); SUFFIX=-model; fi
  python3 tools/conversation/social-routing/evaluate.py "$BIN" "$OUT/$VERSION-fixed-sample$SUFFIX" --sample "${ARGS[@]}"
  # Historical baseline folder says sample but contains full41; preserve name for audit.
  NAME=$VERSION-full
  if [[ "$VERSION" == baseline ]]; then NAME=baseline-sample; fi
  python3 tools/conversation/social-routing/evaluate.py "$BIN" "$OUT/$NAME$SUFFIX" "${ARGS[@]}"
 done
 bash tools/conversation/social-routing/regressions.sh "$BIN" "$VERSION"
done
python3 tools/conversation/social-routing/story_dev.py "$BASELINE" "$BINARY" "$OUT/story-sample" --model "$MODEL" --sample 20
python3 tools/conversation/social-routing/story_dev.py "$BASELINE" "$BINARY" "$OUT/story-full" --model "$MODEL"
sha256sum -c tools/conversation/social-routing/appended-question-frozen.sha256
python3 tools/conversation/social-routing/appended_question.py "$BASELINE" "$OUT/baseline-appended-question"
python3 tools/conversation/social-routing/appended_question.py "$BINARY" "$OUT/candidate-appended-question"
python3 tools/conversation/social-routing/audit.py
python3 tools/conversation/social-routing/state_audit.py "$BINARY"
