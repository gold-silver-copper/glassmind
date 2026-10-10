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
    systemd-run --user --unit=glassmind-loop --collect --same-dir \
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
