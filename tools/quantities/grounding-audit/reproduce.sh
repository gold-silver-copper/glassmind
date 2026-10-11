#!/usr/bin/env bash
set -euo pipefail
# Run from this worktree root, with original read-only mounts still present.
: "${CARGO_TARGET_DIR:?use the existing provided target directory}"
: "${AUDIT_RUN:?choose a new persistent directory outside /tmp}"
case "$AUDIT_RUN" in /tmp|/tmp/*) exit 2;; esac
mkdir -p "$AUDIT_RUN/tmp"
export TMPDIR="$AUDIT_RUN/tmp"
export PYTHONPATH="$(pwd)/../quantities-run/tmp/quantities-deps"
source_dir="$(pwd)/../groundingaudit-source"
owned="tools/quantities/grounding-audit"
exec 2>"$AUDIT_RUN/commands.log"
set -x
bounded() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
python "$owned/prepare.py"
bounded "$source_dir/equation_sequence" "$source_dir/model.json" < "$AUDIT_RUN/replay200-input.jsonl" > "$AUDIT_RUN/replay200-original.jsonl"
bounded "$source_dir/equation_executor" < "$AUDIT_RUN/gold-all-input.jsonl" > "$AUDIT_RUN/gold-all-original.jsonl"
python "$owned/verify.py"
python "$owned/build.py"
bounded cargo build --offline --release --manifest-path "$AUDIT_RUN/helper/Cargo.toml" -j1 > "$AUDIT_RUN/build.log" 2>&1
bounded "$CARGO_TARGET_DIR/release/grounding-audit-helper" "$source_dir/model.json" --free < "$AUDIT_RUN/replay200-input.jsonl" > "$AUDIT_RUN/replay200-helper.jsonl"
python "$owned/verify.py"
bounded "$CARGO_TARGET_DIR/release/grounding-audit-helper" "$source_dir/model.json" --train-oracles < "$AUDIT_RUN/clean.jsonl" > "$AUDIT_RUN/diagnostics.jsonl"
python - <<'PY'
import os,json
from pathlib import Path
r=Path(os.environ['AUDIT_RUN']);rows=[json.loads(l) for l in (r/'clean.jsonl').read_text().splitlines()]
(r/'fit-free-input.jsonl').write_text(''.join(json.dumps(x['question'])+'\n' for x in rows if x['split']=='FIT'))
PY
bounded "$source_dir/equation_sequence" "$source_dir/model.json" < "$AUDIT_RUN/fit-free-input.jsonl" > "$AUDIT_RUN/fit-free-original.jsonl"
bounded cargo test --offline --release --manifest-path "$AUDIT_RUN/helper/Cargo.toml" -j1 -- --test-threads 4 > "$AUDIT_RUN/tests.log" 2>&1
python "$owned/summarize.py"
