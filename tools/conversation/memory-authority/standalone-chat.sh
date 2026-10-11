#!/usr/bin/env bash
# Explicit adapter for replaying the same conversation check against library CLI.
set -euo pipefail
: "${EVERYDAY_BINARY:?Set path to glassmind-everyday}"
[[ "$1" == talk ]]
shift
exec "$EVERYDAY_BINARY" chat "$@"
