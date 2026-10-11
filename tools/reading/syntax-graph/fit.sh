#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?}" "${CARGO_TARGET_DIR:?}"
RUN="$(pwd)/../graphsyntax-run"
SOURCE="$(pwd)/../udsyntax-source"
[[ ! -e "$RUN/candidate/model.bin" && ! -e "$RUN/prefit.json" ]]
python3 - <<'PY'
import hashlib,json,pathlib
paths=[pathlib.Path(p) for p in ['crates/reading/src/syntax.rs','crates/reading/src/syntax_graph.rs','crates/reading/src/bin/reading-syntax.rs','tools/reading/syntax-graph/PLAN.md','tools/reading/syntax-graph/authored.jsonl','../graphsyntax-run/baseline/model.bin']]
pathlib.Path('../graphsyntax-run/prefit.json').write_text(json.dumps({str(p):{'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths},indent=2))
PY
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python3 tools/reading/syntax/measure.py "$RUN/candidate/train-resources.json" "$CARGO_TARGET_DIR/release/reading-syntax" train --train "$SOURCE/en_ewt-ud-train.conllu" --graph-tagger "$RUN/baseline/model.bin" --model "$RUN/candidate/model.bin" --report "$RUN/candidate/train-report.json"
