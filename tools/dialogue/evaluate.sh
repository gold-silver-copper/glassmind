#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?set an on-disk TMPDIR}" "${CARGO_TARGET_DIR:?use existing target directory}"
cap() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
sgd_data="${SGD_ROOT:-/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full}/selected"
sgd_run="$TMPDIR/multi"
for sgd_scope in first multi full; do
  sgd_args=()
  if [[ "$sgd_scope" != full ]]; then sgd_args=(--sample "$sgd_scope"); fi
  for sgd_epochs in 0 12; do
    cap "$TMPDIR/metrics-venv/bin/python" tools/dialogue/evaluate.py \
      --binary "$CARGO_TARGET_DIR/release/dialogue-multi" \
      --model "$sgd_run/model-$sgd_epochs.json" --data "$sgd_data/dev.json" \
      --schema "$sgd_data/dev-schema.json" --output "$sgd_run/$sgd_scope-$sgd_epochs" \
      "${sgd_args[@]}" > "$sgd_run/$sgd_scope-$sgd_epochs.log"
  done
  python - "$sgd_run" "$sgd_scope" <<'PY'
import json,pathlib,sys
p=pathlib.Path(sys.argv[1]);scope=sys.argv[2]
for e in (0,12):
 r=json.loads((p/f'{scope}-{e}/report.json').read_text())['all']
 print(scope,e,{k:r[k] for k in ['dialogues','user_turns','gold_frames','active_f1','strict_frame_correct','full_joint_correct','fuzzy_joint_sum','gold_switches','correct_switches']})
PY
done
