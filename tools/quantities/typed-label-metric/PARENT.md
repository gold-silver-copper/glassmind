# Independent review: accepted metric, separate rebaseline

The exact-label-comparison-v3 contract passes independent review. This accepts
the offline measurement definition and baseline, not the rejected rate solver.
Historical legacy scores and the v2 rejection are unchanged.

The parent verified all 24 manifest entries, reproduced 12 sample/full files
byte-for-byte, checked every row and all five changed answer grades, and passed
15 test groups plus 330 additional contract, conversion, malformed-container and
Unicode checks. [Detailed review](PARENT.json) and the separate parent probe
reports retain the evidence. The quantity sources are unchanged from the frozen
baseline commit `687f2751ff256f2071330590345ef0652fb09baa`.

Baseline: **22 correct label matches, 4,450 refusals, 3 wrong, 0 unscorable
answers / 4,475**. The frozen rejected candidate scores 28/4444/3/0 under this
new definition; it remains rejected and receives no capability-gain point.

Bare numeric labels compare coefficients only: requested units and prediction
dimensions are not checked. Explicit unit labels require compatible explicit
prediction units and exact canonical values. Neither case establishes that a
dataset label correctly answers its question. This is not an official MathQA
metric. Unsupported labels stay in the denominator, with answered unsupported
labels reported separately and also counted as wrong in conservative totals.

Reproduce from the repository root using the retained public data and frozen
outputs; no inference or training is performed:

```sh
WORK=/home/gsc/.cache/glassmind/continuous-20261010
PYTHONDONTWRITEBYTECODE=1 python tools/quantities/typed-label-metric/reproduce.py \
  --data "$WORK/quantities-run/tmp/mathqa" \
  --deps "$WORK/quantities-run/tmp/quantities-deps" \
  --frozen "$WORK/typedmetric-run/baseline" \
  --output "$WORK/coordinator/typedmetric-review"
PYTHONDONTWRITEBYTECODE=1 python tools/quantities/typed-label-metric/independent_review.py \
  --reports "$WORK/coordinator/typedmetric-review/full" \
  --output "$WORK/coordinator/typedmetric-review/arithmetic.json"
```

Future solver experiments must declare their acceptance criteria prospectively
against this separate baseline and retain the legacy comparison as a diagnostic.
