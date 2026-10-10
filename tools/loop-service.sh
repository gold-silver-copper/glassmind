#!/bin/sh
# Runs the improvement loop as a detached, resumable systemd user service.
#   tools/loop-service.sh start STATE_DIR [LOOP_BINARY]
#   tools/loop-service.sh stop STATE_DIR     (finishes the current round, then stops)
#   tools/loop-service.sh status STATE_DIR
# The loop resumes from STATE_DIR wherever it stopped.
set -e
cmd=$1
state=$(realpath "$2")
bin=${3:-$(dirname "$0")/../target/release/glassmind-loop}
case "$cmd" in
  start)
    rm -f "$state/STOP"
    bin=$(realpath "$bin")
    # Its own slice with a memory cap: if the loop and its agents outgrow it,
    # they are slowed and then killed inside the slice, never the terminal.
    systemd-run --user --unit=glassmind-loop --collect --same-dir \
      --slice=glassmind.slice -p MemoryHigh=28G -p MemoryMax=36G -p MemorySwapMax=0 \
      -p OOMScoreAdjust=800 \
      --setenv=PATH="$PATH" --setenv=HOME="$HOME" \
      "$bin" run --state "$state"
    ;;
  stop) touch "$state/STOP" ;;
  status)
    systemctl --user --no-pager status glassmind-loop 2>/dev/null | head -3 || true
    "$(realpath "$bin")" status --state "$state" | tail -15
    ;;
  *) echo "usage: $0 start|stop|status STATE_DIR [LOOP_BINARY]" >&2; exit 2 ;;
esac
