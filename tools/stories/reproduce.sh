#!/usr/bin/env bash
# Training members only. No API, external solver, or held-out evaluation.
set -euo pipefail
cd "$(dirname "$0")/../.."
export TMPDIR="$PWD/scratch/stories/tmp"
export CARGO_TARGET_DIR="$PWD/.cache/stories-target"
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$TMPDIR"
result_dir=$(mktemp -d "$PWD/scratch/stories/reproduce.XXXXXX")

# --wait preserves the child exit status when setsid must fork.
run_scoped() {
    local name=$1
    shift
    systemd-run --user --scope --quiet --collect -pMemoryMax=8G -pMemorySwapMax=0 \
        setsid --wait timeout --kill-after=5s 600 "$@" >"$result_dir/$name.log" 2>&1
}

python3 tools/stories/babi.py prepare --provenance "$result_dir/source.json" >"$result_dir/prepare.log"
run_scoped build cargo build -p glassmind-stories -j 1
run_scoped clippy cargo clippy -p glassmind-stories --all-targets -j 1 -- -D warnings
run_scoped train python3 tools/stories/babi.py train --model "$result_dir/action-model.json"
test -s "$result_dir/action-model.json"
run_scoped sample python3 tools/stories/babi.py eval --model "$result_dir/action-model.json" \
    --sample 20 --report "$result_dir/sample.json"
run_scoped dev python3 tools/stories/babi.py eval --model "$result_dir/action-model.json" \
    --sample 0 --report "$result_dir/dev.json"
python3 - "$result_dir" <<'PY'
import json
from pathlib import Path
import sys
root = Path(sys.argv[1])
for name in ("sample", "dev"):
    report = json.loads((root / f"{name}.json").read_text())
    print(name, report["total"], report["totals"])
print("Artifacts:", root)
PY
