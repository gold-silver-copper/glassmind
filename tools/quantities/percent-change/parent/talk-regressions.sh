#!/usr/bin/env bash
set -euo pipefail
R=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/percentchange-review
B="$R/glassmind-integrated"
M=/home/gsc/.cache/glassmind/models/everyday/intent.json
python tools/talk-dev.py "$B" --model "$M" --output "$R/calc27.json"
python tools/talk-story-dev.py "$B" --model "$M" --output "$R/story47.json"
python tools/facts/sets/contracts.py --binary "$B" --output "$R/facts36"
python tools/facts/sets/session_contracts.py --binary "$B" --bundle "$R/facts36/bundle" --output "$R/facts-session16"
python tools/conversation/social-routing/appended_question.py "$B" "$R/stale"
