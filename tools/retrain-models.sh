#!/bin/sh
# Retrains glassmind's small models from what it already gets right: the
# loop's last accepted outcomes on the training set (fixes count once the
# scorer ran them). Thresholds stay those calibrated on the dev set; check
# the dev set (and the selection set) before committing the new models.
#
#     tools/retrain-models.sh STATE TRAIN_DATA WORDNET_XML WIKIDATA_JSON...
#
# STATE is the loop's state directory, TRAIN_DATA the directory with the
# training inputs.jsonl and labels.jsonl.
set -eu
state=$1 data=$2 wordnet=$3
shift 3
here=$(cd "$(dirname "$0")/.." && pwd)
work=${TMPDIR:-$HOME/.cache/claude-tmp}/retrain-models
mkdir -p "$work"
python3 -c '
import json, sys
for o in json.load(open(sys.argv[1]))["train"]:
    print(json.dumps(o))
' "$state/baseline.json" > "$work/outcomes.jsonl"
"$here/target/release/glassmind" train-intents --inputs "$data/inputs.jsonl" \
    --outcomes "$work/outcomes.jsonl" --labels "$data/labels.jsonl" \
    --out "$here/crates/english/models/intents.tsv"
wikidata=""
for w in "$@"; do wikidata="$wikidata --wikidata $w"; done
# shellcheck disable=SC2086
python3 "$here/tools/datagen/wordnet_bridge.py" --senses 0 --wordnet "$wordnet" $wikidata \
    --model "$here/crates/english/models/intents.tsv" --out "$here/crates/english/models/synonyms.tsv"
