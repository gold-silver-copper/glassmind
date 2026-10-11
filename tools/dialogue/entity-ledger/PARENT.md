# Independent rejection of the system-entity ledger

All 16 frozen calibration settings fail. The strictest produces 474 correct
new scored slot-turn fills, 41 newly wrong claims, 150 unscored held changes,
and loses two previously correct joint turns and frames. Production dialogue
code and its approved model remain unchanged. No candidate DEV evaluation ran.

The parent independently joined original TRAIN annotations with all 3,174
utterance-only baseline inputs, checked the 642-conversation split, reconstructed
128 FIT-selected role labels, and verified every one of 51,647 candidate scores
and source spans. All 22,912 decisions match the native choices. A separate
Python replay reproduced every grid's complete persistent state overlay, strict
frame/joint counts, initial events and losses. It did not use worker correctness
flags or derived gold labels to score claims. Maximum rank-score reconstruction
error from f64 versus native f32 accumulation was 0.000162.

This was an offline overlay on frozen predictions, without feeding proposed
values back into existing reference mechanisms. It is sufficient to fail the
declared prerequisite; it is not an integrated runtime or DEV result. No retrain,
threshold extension, relaxed gate or candidate DEV inference followed rejection.
The prewritten fresh-entity probes were not run against a candidate, because no
integrated candidate reached their prerequisite. Their approved baseline remains
0/12 supplied destinations and four unchanged negative controls; immediate
requests also expose an existing routing limitation.

Original submission: 114 files / 223,211,085 bytes, preserved and verified before
review. The 70-member pre-rollback manifest and all 11 supplied source identities
were verified independently, and the rebuilt baseline predictions are identical
for all 3,174 TRAIN conversations. Thirteen original source files match both the
restored worktree and production. Archives were reverified before removing the
owned worktree and temporary login link. Raw evidence remains under
`~/.cache/glassmind/continuous-20261010/entityledger-run/terminal-originals`.

A subsequent FIT-only diagnostic deduplicates 13,425 extracted source entities.
Restricting role owners to the old router's active services would remove 1,648
incorrect role labels, but also 8,135 exact role labels. Among entities whose
annotated owner is represented by that router, only eight role errors remain.
This does not support treating the narrow router as general source authority.
A future experiment needs better source context and unknown-role handling while
retaining useful entities from other services, rather than discarding them to
make the gate easier. No model or prediction changed in this diagnostic.

SGD revision `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0;
zero new downloads. Data-derived evidence retains that license. Worker usage:
2,271,706 input tokens including 2,164,992 cached, plus 33,087 output = 2,304,793.
Root usage is not included. The new source representation is rejected; earlier
USER-span failures and the unchanged 610/1,119 DEV score remain distinct evidence.
