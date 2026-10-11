#!/bin/sh
# glassmind as a rustbench agent (github.com/gold-silver-copper/rustbench):
# acts on the request in RB_WORK, or writes its reply to .rustbench-reply.
#   rustbench run --agent "GLASSMIND=/path/to/glassmind tools/rustbench-agent.sh"
exec "${GLASSMIND:-glassmind}" do "$RB_WORK" --reply-file "$RB_WORK/.rustbench-reply" "$(cat "$RB_PROMPT_FILE")"
