# One frozen SQuAD2 null-selection experiment

Written before runtime edits, training, CAL or candidate DEV. Base source
80b9f636050accfbe6ca0fbdb0ecc2bc6ca531d9. Original v1 baseline all11873:
433 substantive correct, 5140 expected refusals, 5064 unsupported, 1236 wrong;
official EM46.946854207%, F147.435937977%; always-refuse5945/11873.

Hypothesis: genuine unanswerable plus wrong frozen-span TRAIN negatives improve
answer-versus-null selection beyond v1 margin while retaining useful answers.
This is a proposal selector, never entailment or factual verification. Prior
support/roles rejection reports were read: overlap and quote presence do not
prove roles, predicates or negation. First12 SHA-ID FIT rows and natural class
counts inspected (report); no labels used to choose features beyond this audit.

Selection: map original80 v1 FIT/CAL article context hashes to TITLE using only
v1 TRAIN and original selection.json. Exclude these titles from v2 TRAIN.
Sort remaining articles SHA256(title); first96 FIT, next32 CAL. Remove exact
contexts shared across FIT/CAL/DEV from TRAIN sides before sampling. Assert title,
context and ID disjointness; no dev-outcome selection. Sort questions SHA256(id),
first8000 FIT, first2000 CAL; retain natural distribution. Labels offline only.

Frozen v1 model SHA b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5.
Generate one raw best span/score/margin/evidence per row with this model. The
optional selector may revive margin-refused proposals, but not proposals with
no lexical support, invalid offsets or input-limit/nonfinite failures. It can
only select the exact original span or refuse. Absent optional model, original
v1 code path/output/model are unchanged. Runtime inputs deny unknown fields.

Exact representation: sparse 8192-dimensional FNV1a-hashed linear features;
parent QA tokenization and stopword set, lowercase, suffix stemming ing/ed/es/s
only when stem >=3 chars. Bias; clipped score/10 and margin/2; margin buckets
<.05,<.1,<.2,<.4,other; question/evidence/answer token lengths (caps64/128/10);
answer fraction capitals/digits, first-token capital/digit; unique content-anchor
stem coverage in context/evidence/outside answer, missing capital/numeric anchors;
mean nearest-anchor distance clipped32, before/after/inside anchor fractions,
first/last anchor side, ordered adjacent matched-anchor fraction;
question/evidence negation presence and disagreement; repeated exact span count
clipped4. All numeric features additionally crossed with first question word.
Lexical sparse features: question content stems marked evidence-present/missing,
matched question stems crossed with candidate side (before/inside/after), and
question first word crossed with immediate candidate left/right token stems.
No IDs, titles, labels, answer tables, embeddings, pretrained models or runtime LLM.
Feature values/weights bounded and finite; model version1, exact8192 weights.

One schedule: zero-init binary logistic, 12 passes in SHA-ID order, online
AdaGrad learning rate .08, accumulator initial1, per-example shrink on active
weights gradient += .0001*w. No class balancing/resampling. Positive only a
nonempty normalized raw span officially exact against nonempty-normalized gold
on an answerable row; all genuine unanswerables and wrong spans negative.
Save FIRST fitted weights before any CAL, never retrain to recover lost weights.

CAL select maximum coverage threshold over complete tied score groups with
substantive exact precision>=60%, >=100 correct substantive answers, and strict
correct (substantive+expectedrefusals)>always-refuse correct. No passing threshold:
STOP REJECTED, no candidate DEV, retain weights/hash and original patch then
restore runtime. Keep complete frontier. No schedule/threshold rescue.

If CAL passes, freeze source/model/threshold before SHA-ID first200 DEV then
all11873 DEV. Final gate: substantivecorrect>=433, wrong<1236, precision>=50%,
strictcorrect>5945, officialEM>=50.5%; no annotation leakage or broken v1/provenance.
Report official EM/F1 separately from strict counts; nonempty 'A' normalizing
empty remains wrong on impossible examples. Every ID counted. Save predictions
and transitions; scoring fixes reuse them. Either gate failure restores runtime.

Independent authored probes in probes.json are frozen before fitting; role,
predicate, negation, ambiguity, Unicode, no-support each has positive counterpart.
Report raw/selected failures, no semantic-proof claim. Tests: original v1 behavior,
serialization/finite/model/input bounds, unknown-field rejection, UTF8/repeated
spans. Full SQuAD1 preserved by unchanged default implementation; optional exact
saved-output replay if available within budget. No main CLI/cross-track changes.
