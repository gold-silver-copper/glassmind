#!/usr/bin/env bash
set -euo pipefail
RUN=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/identifierroles-review
export TMPDIR="$RUN/tmp" GLASSMIND_SCRATCH="$RUN/scratch" CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/identifierroles-run/target CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=4
mkdir -p "$TMPDIR"
python tools/coding-identifier-roles/evaluate.py --binary "$RUN/glassmind-integrated" --output "$RUN/integrated-sample" --sample > "$RUN/sample.log" 2>&1
python tools/coding-identifier-roles/evaluate.py --binary "$RUN/glassmind-integrated" --output "$RUN/integrated-full" > "$RUN/full.log" 2>&1
python tools/coding-identifier-spelling/probes.py "$RUN/glassmind-integrated" "$RUN/spelling-integrated" > "$RUN/spelling.log" 2>&1
python - "$RUN" <<'PY'
from pathlib import Path
import json,sys
r=Path(sys.argv[1]);a=json.loads((r/'integrated-full/results.json').read_text());b=json.loads((r/'spelling-integrated/report.json').read_text());assert len(a)==42 and all(x['pass_'] for x in a);assert b['safe']==8
old=json.loads(Path('tools/coding-identifier-roles/reports/candidate-v2-full.json').read_text());assert len(old)==42
for x,y in zip(old,a,strict=True):
 for k in ['id','request','pass_','changed','exact','behavior','actual']:assert x[k]==y[k],(x['id'],k)
 assert x['result']['stdout']==y['result']['stdout'],x['id']
print('42 frozen cases reproduce worker outcomes/source exactly; all8 spelling cases safe')
PY
