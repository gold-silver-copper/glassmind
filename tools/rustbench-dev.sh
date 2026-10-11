#!/bin/sh
# Full dev split, with a cap and a process-group timeout around every cargo run.
# Usage: GLASSMIND=/absolute/path/to/glassmind tools/rustbench-dev.sh [OUT_DIR]
set -eu
project=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bench=${RUSTBENCH_REPO:-$HOME/code/rustbench}
out=${1:-$HOME/.cache/glassmind/rustbench-dev}
mkdir -p "$out/tmp" "$out/bin"
out=$(realpath "$out")
export TMPDIR="$out/tmp"
export GLASSMIND=${GLASSMIND:-$project/target/release/glassmind}
export GLASSMIND_REAL_CARGO=$(command -v cargo)
cat > "$out/bin/cargo" <<'CARGO'
#!/bin/sh
set -eu
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=4
exec systemd-run --user --scope --quiet --collect \
  -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 80 "$GLASSMIND_REAL_CARGO" "$@"
CARGO
cat > "$out/bin/agent" <<'AGENT'
#!/bin/sh
set -eu
exec setsid --wait timeout --kill-after=10s 100 "$GLASSMIND" do "$RB_WORK" \
  --reply-file "$RB_WORK/.rustbench-reply" "$(cat "$RB_PROMPT_FILE")"
AGENT
chmod +x "$out/bin/cargo" "$out/bin/agent"
export PATH="$out/bin:$PATH"
# No held-out task content is printed or selected by this entry point.
systemd-run --user --scope --quiet --collect \
  -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 3600 \
  "$bench/target/release/rustbench" --tasks "$bench/tasks" run \
  --split dev --jobs 1 --agent-timeout 120 --agent "$out/bin/agent" \
  --scratch "$out/scratch" --out "$out/results.jsonl" > "$out/run.log" 2>&1
"$bench/target/release/rustbench" report "$out/results.jsonl" | tee "$out/totals.txt"
