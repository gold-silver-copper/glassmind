# TRAIN facts selection diagnosis

No inference, model fitting, new index, downloads or DEV inspection in this
step. Verify retained TRAIN labels, question-only inputs, baseline predictions,
TRAIN999 index and original selection hashes before joining them.

Measure forward (P) versus inverse (R) anchor coverage and strict entity/relation/
answer outcomes on the old selection-eligible FIT rows. Decompose answered
wrong-anchor rows by whether the annotated anchor exists and whether its alias
is present. Count alias-link failures separately from missing/nonunique claims.
Report the existing selected IDs' forward/inverse membership and frequency
selection effect. Labels are offline diagnostics, never stored claims.

Use this evidence to choose a new hypothesis; do not repeat the known failed
interrogative-boundary rule or partial incoming-edge acquisition. Original
TRAIN999 is still rejected, historical500 remains a separate seeded series.
No accuracy change is claimed by this diagnostic. Existing model used TRAIN;
these are in-sample structural counts. All original artifacts remain immutable.
