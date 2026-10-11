#!/usr/bin/env bash
# Training members only. No API, external solver, or held-out evaluation.
set -euo pipefail
cd "$(dirname "$0")/../.."
export TMPDIR="$PWD/scratch/stories/tmp"
export CARGO_TARGET_DIR="$PWD/.cache/stories-target"
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$TMPDIR"
result_dir=$(mktemp -d "$PWD/scratch/stories/reproduce.XXXXXX")

# setsid can fork when launched as a process-group leader. Wait for the entire
# named scope before consuming its output or starting another compilation.
run_scoped() {
    local name=$1
    shift
    local unit="glassmind-stories-${name}-$$"
    local log="$result_dir/$name.log"
    systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
        --unit="$unit" setsid timeout --kill-after=5s 600 "$@" >"$log" 2>&1
    local count=0
    while [[ $(systemctl --user show "$unit.scope" -p ActiveState --value) =~ ^(active|activating|deactivating)$ ]]; do
        sleep 0.2
        count=$((count + 1))
        if ((count > 3050)); then
            systemctl --user stop "$unit.scope"
            cat "$log"
            return 1
        fi
    done
    if [[ $name == build || $name == clippy ]]; then
        # Do not mistake the early exit of a forking setsid for cargo success.
        if ! rg -q 'Finished .* profile' "$log"; then cat "$log"; return 1; fi
    fi
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
