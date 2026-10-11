#!/usr/bin/env bash
set -euo pipefail
OWN=$(cd -- "$(dirname -- "$0")" && pwd)
WORK=/home/gsc/.cache/glassmind/continuous-20261010
DATA=${MATHQA_DATA:-$WORK/quantities-run/tmp/mathqa}
FROZEN=${MATHQA_FROZEN:-$WORK/mathmetric-run/baseline}
DEPS=${MATHQA_DEPS:-$WORK/quantities-run/tmp/quantities-deps}
OUT=${1:-$OWN/reproduced}
export PYTHONPATH="$DEPS"
export PYTHONDONTWRITEBYTECODE=1
python -m unittest discover -s "$OWN" -v
python "$OWN/audit.py" --data "$DATA" --frozen "$FROZEN" --output "$OUT/sample" --sample
python "$OWN/audit.py" --data "$DATA" --frozen "$FROZEN" --output "$OUT/full"
python "$OWN/review.py" "$OUT/full"
