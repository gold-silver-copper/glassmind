# Mention selection needs evidence that the full name is present

The exact-alias candidate study found a useful upper bound: 1,400 unique TRAIN-FIT
questions contain a correct eligible alias that the native longest-match rule
misses. A learned selector based only on grammatical context and span shape did
not meet its frozen reliability requirement. Neither study changes production
linking, QA defaults or answering scores.

Preparation excluded all 1,949 existing CAL indices and 67 additional questions
sharing their native normalized text. The remaining 17,465 rows form 17,291 groups.
All candidate lists independently reconstruct the native link result exactly.
The 160,285 matched span occurrences include 135,304 ambiguous occurrences; those
never become eligible. There are 9,085 groups with a correct unique, case-supported
candidate, including 1,400 missed by the baseline. Candidate enumeration retains
repeated positions and preserves native acronym and tie behavior.

The single learned experiment uses a 157,357-byte native sparse margin model,
trained for 20 epochs on 19,719 eligible spans from the training groups. Inputs
contain three positions of grammatical context on each side, a masked question
prefix, and span shape/position. Names, span content, entity IDs, answer IDs and
relation labels are excluded. Fixed function-word vocabulary maps other words to
OTHER. Internal validation holds out normalized-question groups by hash, giving
3,598 rows in 3,567 groups. Conflicting annotation groups remain included.

The predeclared 55 threshold pairs choose when to replace the native baseline
with a confident alternative; uncertain predictions preserve that baseline.
No setting delivers the required 50 net correct entity links with zero newly
wrong links and zero losses of correct links. The highest-gain diagnostic setting
adds 219 correct links but introduces 50 new errors, including six formerly
correct links. The best setting with no new errors adds only five. No setting is
selected, and no CAL, DEV or full-QA evaluation follows this rejection.

Of 44 unlinked-to-wrong cases at the diagnostic setting, 41 lack any eligible
correct candidate. The model often accepts a partial name/title or a predicate
word when the intended entity is missing or ambiguous. Three have a correct
candidate, as do all six lost-correct cases. These observations support studying
complete mention boundaries and explicit absence evidence in a future experiment;
they do not justify tuning the rejected model's thresholds or current data split.

The preparer completed in 44.011 seconds; the fit/score/report run took 2.001
seconds. Both passed their frozen resource caps. Native contracts cover longest
alternatives, alias ambiguity, acronym fallback blocking, repeated mentions,
Unicode spans, name/ID feature invariance, case eligibility, entity aggregation
and nonfinite scores. No model workers, credentials, downloads or official
held-out data were used. The one fitted model and all original candidates,
features, scores and grid outcomes remain external, with committed hash manifests.

The original fullgraph QA promotion result remains rejected at 554/1,949 with
29 newly wrong cases. This study concerns entity linking on FIT-internal data;
it cannot rewrite that outcome or establish independent generalization.
