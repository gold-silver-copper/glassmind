# Full public claims snapshot: frozen acquisition and storage plan

Frozen 2026-10-11 before downloading any of the remaining 85 shards.
The prior single-shard QA CAL experiment failed its unchanged promotion gate.
Hypothesis: ingesting the complete independently published claims release can
remove the deliberately partial graph as a coverage bottleneck. This experiment
first establishes storage correctness and feasibility; it does not claim QA gain.

Select ALL 86 files named all/part-NN-of-86.parquet from the already pinned
permutans/wikidata-claims revision39e6837bef8b6f982fbf23acd19134e882400db7,
CC0-1.0, Wikidata20260928, excluding scholarly works as the publisher documents.
Pinned tree SHA256:985454279e62e90ea5373e5f349195ee31f64829210a9ce8ffa0e9ba134005f2.
Expected raw bytes44,633,986,614; publisher states789,435,443 source rows.
First shard529,038,605bytes already retained and hash verified. All85 remaining
shards44,104,948,009bytes are selected by public file inventory, never QA IDs.
Keep the old pilot/read-only source artifacts intact. No new names needed.

Acquisition gates: at most96GiB received including retries; 7200seconds total;
60second request inactivity timeout; at most2attempts/file; honor Retry-After
up to60seconds and stop if longer/unknown. Sequential unauthenticated requests,
SHA256 and exact bytes checked against publisher inventory before use. Retain
failed partials. Source data, temporary files and databases stay outside git.
No OpenAI/API/other-provider/model calls. Check disk and memory before launching.

Projection: stream bounded PyArrow25.0.1 batches of source rows into a separate
claims-only SQLite projection, preserving global raw row ordinal+1 across files.
Select the same Q-to-Q, P-labelled, normal/preferred, value-bearing statements
as the pilot. Preserve GUID, rank, qualification flag and raw file provenance.
Check canonical numeric ID round trips for every retained claim; malformed
eligible records fail, never silently repair. Keep exact per-file exclusion,
rank, qualifier and source row accounting. Raw Parquet retains full qualifiers
and references; graph edges do not imply timeless/current truth.
Projection max96GiB,14400seconds under8GiB memory/no swap. No secondary indexes
needed in this temporary numeric projection. Pin its final hash and report.

Native import: separate format/provenance for the broader graph. Reuse validated
Rust alias keys and provenance from the immutable native pilot (SHA256
f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b;
12,306,595,840bytes), copying into a new output. Claims come exclusively from the
new pinned projection; do not blend old QA-derived graphs. Preserve all statement
witnesses, deduplicate exact triples, include claim subjects without names, build
forward/inverse/name indexes and check full integrity/references before exposing
a readable header. Existing format1/2 behavior must remain compatible.
Native output max192GiB,21600seconds under8GiB/no swap. Combined added run bytes
(including temporary/sort files, projection, raw files and failed attempts) must
stay below384GiB. Abort bounded jobs on limit violations; do not enlarge failed
gates after seeing results. Existing old artifacts are excluded from added bytes.

Storage acceptance: every source byte hash matches; exact source/projection/native
accounting; deterministic raw positions 0,last,and every100003rd row in EACH file
checked against projection and native witnesses; native indexed query plans;
positive forward/inverse query checks from retained raw witnesses independent of
QA; existing old graph CLI/regression checks and workspace required gates pass.
Report actual timings, disk and memory, without extrapolating a small query batch
to broad latency. No promotion/default change from successful storage alone.

After storage passes, freeze a separate QA comparison using the existing relation
model and question-only inference before annotation joins. Preserve strict joint
metrics, fixed denominators and regressions. Do not inspect official held-out or
run DEV unless a prospectively fixed CAL gate passes. No QA examples or IDs enter
acquisition, projection, import or source correctness sampling.
