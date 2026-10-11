# Work rates rejected: insufficient DEV gain and independent language coverage

Do not integrate this runtime patch. The frozen gate required five additional
correct MathQA DEV answers; the candidate adds four. An independent coordinator
replay answers none of 17 solvable, differently worded questions. All 13
underdetermined/contradictory probes refuse, with no wrong answers. The arithmetic
representation works on its authored grammar, but the independent language
coverage does not support shipping it. No post-DEV rescue or threshold change.

The parent independently recomputed all 4,475 grades using the unchanged frozen
v3 metric, verified every question hash/order and all 32 old answer payloads,
and separately checked the four new answers by summing reciprocal rates from the
question text. V3 is 33 correct /4,439 refused /3 wrong versus 29/4,443/3. Legacy
is 32/4,439/4; its additional mismatch is an explicit `2 hours` label. The metric
was not changed to rescue this result. Three of the four gains share a coordinated
three-worker completion-time form; one uses three filling pipes. No staged DEV
gain was observed. Full inference was not repeated for this independent audit.

The 30 parent probes were frozen before candidate inspection. Before any native
execution, a draft label was corrected: two known tank capacities and completion
times determine a volume-normalized rate, so that case is solvable (6/5 hours),
not an expected refusal. A slowdown negative was clarified to say the slowdown
is unknown. Original drafts, the corrected freeze, baseline and candidate outputs
are retained in [parent](parent/). This correction was independent of candidate
behavior. All 30 baseline and candidate inputs match the final frozen cases.

Verified: 19 coordinator archive entries (75,781 bytes); 15 original pre-rollback
entries (103,496 bytes); 26 raw artifacts (29,276,066 bytes); 12 frozen source,
probe, adapter and binary identities. All original report copies and raw artifacts were reverified before removing the
inactive worker checkout and temporary login link. The worker restored all quantity runtime
files, and the parent checked them against source65e7240 and current root bytes.
The new module/test is absent from the production tree. The approved quantity
runtime and 29/4,475 score remain unchanged; no main talk gain is claimed.

The original [handoff](HANDOFF.md), [frozen plan](PLAN.md), [rejected patch](candidate.patch)
and [independent full audit](parent/full-audit.json) remain intact. Worker tests,
33 authored cases, all3,116 synthetic outcomes and existing arithmetic regressions
passed; initial clippy/probe failures remain archived. Those tests do not override
the failed coverage gates. Raw artifacts remain at
`/home/gsc/.cache/glassmind/continuous-20261010/workrates-run`; original runtime
source is in `submitted`, final reports in `parent-submitted`.

No downloads. Reused MathQA revision fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6,
Apache-2.0; TRAIN9,013,733 bytes and DEV1,350,141 bytes. The worker disclosed that
reading the source card displayed an unrelated annotated combinatorics example.
It reports not using that example; do not claim zero annotation exposure. Parquet
implementation reads used Problem only; scoring read options/correct after frozen
predictions. The original trace and report retain this deviation.

Further work should change clause composition and independently measure ordinary
wording coverage before another full DEV run. Extending the same small template
list against these evaluation sentences is not an accepted follow-up.
