#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
: "${TMPDIR:?persistent directory required}"
: "${CARGO_TARGET_DIR:?existing target required}"
: "${REPAIR_RUN:?own external run directory required}"
export TMPDIR CARGO_TARGET_DIR
bounded() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
sha256sum -c tools/coding-repair-veto/FROZEN.sha256
bounded cargo fmt --all --check
bounded cargo test -p glassmind -p glassmind-english-rust -j1 -- --test-threads 4
bounded cargo clippy -p glassmind -p glassmind-english-rust --all-targets -j1 -- -D warnings
bounded cargo build -p glassmind --release -j1
# Public request files supplied by coordinator, with provenance and hashes in handoff.
python3 - "$REPAIR_RUN" <<'PY'
import json,pathlib,sys
r=pathlib.Path(sys.argv[1]);rows=[]
for group,path in [('authored',pathlib.Path('tools/coding-repair-veto/contrasts.tsv')),('old68',r/'oldcases/cases.tsv'),('legacy15',r/'oldcases/public-legacy.tsv'),('train172',r/'training-requests.tsv')]:
 for i,line in enumerate(path.read_text().splitlines()):
  label,text=line.split('\t',1);rows.append(dict(group=group,index=i,expected=label,text=text))
(r/'typing-input.json').write_text(json.dumps(rows))
PY
export REPAIR_VETO_INPUT="$REPAIR_RUN/typing-input.json"
REPAIR_VETO_OUTPUT="$REPAIR_RUN/baseline-optimized.json" bounded python3 tools/coding-repair-veto/typing.py "$REPAIR_RUN/baseline-optimized" --baseline
REPAIR_VETO_OUTPUT="$REPAIR_RUN/candidate-optimized.json" bounded python3 tools/coding-repair-veto/typing.py "$REPAIR_RUN/candidate-optimized"
python3 tools/coding-repair-veto/summarize.py "$REPAIR_RUN/baseline-optimized.json" "$REPAIR_RUN/candidate-optimized.json" > "$REPAIR_RUN/typing-summary.json"
mkdir -p "$REPAIR_RUN/frozen"
cp "$CARGO_TARGET_DIR/release/glassmind" "$REPAIR_RUN/frozen/glassmind"
cp crates/cli/src/coding.rs "$REPAIR_RUN/frozen/coding.rs"
sha256sum "$REPAIR_RUN/frozen/"* tools/coding-repair-veto/*.py > "$REPAIR_RUN/frozen.sha256"
python3 tools/coding-repair-veto/native.py "$REPAIR_RUN/baseline/glassmind" "$REPAIR_VETO_INPUT" "$REPAIR_RUN/baseline-native-sample" --sample
python3 tools/coding-repair-veto/native.py "$REPAIR_RUN/frozen/glassmind" "$REPAIR_VETO_INPUT" "$REPAIR_RUN/candidate-native-sample" --sample --confirm
python3 tools/coding-repair-veto/native.py "$REPAIR_RUN/frozen/glassmind" "$REPAIR_VETO_INPUT" "$REPAIR_RUN/candidate-native-full"
# Supplemental contrasts were authored after initial implementation; report separately.
python3 - "$REPAIR_RUN" <<'PY'
import json,pathlib,sys
rows=[]
for i,line in enumerate(pathlib.Path('tools/coding-repair-veto/supplemental.tsv').read_text().splitlines()):
 label,text=line.split('\t',1);rows.append(dict(group='supplemental',index=i,expected=label,text=text))
(pathlib.Path(sys.argv[1])/'supplemental.json').write_text(json.dumps(rows))
PY
REPAIR_VETO_INPUT="$REPAIR_RUN/supplemental.json" REPAIR_VETO_OUTPUT="$REPAIR_RUN/supplemental-baseline.json" bounded "$REPAIR_RUN/baseline-optimized/typing" --test-threads 4
REPAIR_VETO_INPUT="$REPAIR_RUN/supplemental.json" REPAIR_VETO_OUTPUT="$REPAIR_RUN/supplemental-candidate.json" bounded "$REPAIR_RUN/candidate-optimized/typing" --test-threads 4
python3 tools/coding-repair-veto/native.py "$REPAIR_RUN/frozen/glassmind" "$REPAIR_RUN/supplemental.json" "$REPAIR_RUN/supplemental-native"
