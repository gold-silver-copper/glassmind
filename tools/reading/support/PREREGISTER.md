# Frozen experiment: support-v1 (2026-10-10)

Before implementation or dev inspection. Base 1003add43fe9e1970ee0a5380fc15c7304f94aa1.
Baseline supplied: raw 2532/10570; selected 815 right, 791 wrong, 8964 refused.
TRAIN audit: next unused 96 article hashes contain 8000 selected rows with varied
question kinds (3279 what, 894 who, 724 how, 558 when, 335 where). First 16 rows
show inflected relations, competing entities/dates, and lexical paraphrases.
Source occurrence alone is not support.

Hypothesis: an inspectable small tree ensemble over bounded answer shape,
question kind, stem overlap, ordered anchor alignment and local clause features
improves correct coverage over margin alone at comparable selected precision.
No entity/predicate/answer vocabulary, IDs, titles or annotations at runtime.

Split by existing SHA256(contexts joined by newline) article ordering: exclude
all original 64 fit and 16 calibration articles using baseline selection.json;
next96 support-fit capped8000 by SHA256(questionID), next32 support-cal capped2000.
Frozen v1 predictions generate offline exactness labels. Fit only old selected
rows. Features fixed before dev. Boosted squared-error regression trees: 60 trees,
maximum depth 2, minimum leaf 15, learning rate .1, fixed quantile cutpoints
from FIT only (15 interior quantiles). No hyperparameter search.
Threshold: maximum calibration coverage at >=.90 observed exact precision,
>=30 accepts, include all score ties; absent qualifying threshold means failure.
Margin-only selector uses identical calibration target and old-selected rows.
These scores and threshold selections are not calibrated probabilities.

Evaluate fixed SHA256(ID) first200, then all10570 once frozen, without retuning.
Accept only >=400 correct, >=.80 exact precision, more correct than margin-only
at precision no more than .02 lower, unchanged raw proposals/old refusals and
no answer-text changes. Independent novel contrasts cover subject/object roles,
unrelated predicates, negation, answer kinds and changed relations. Report all
failures; do not claim relation verification or automatic factual integration.
If calibration or dev acceptance fails, reject and retain source patch/evidence,
with no deployed support integration. No alternate experiment this batch.
