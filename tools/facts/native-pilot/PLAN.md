# Native ingestion of the independent Wikidata pilot

Frozen2026-10-11 before transformation or QA evaluation. Base91d702b, root-owned
worktree WORK/wikidata-native; WORK=/home/gsc/.cache/glassmind/continuous-20261010.
The preceding native-store compatibility gates passed. This phase must turn the
independent source into a queryable native graph, rather than stop at parity on
999old entities. No model workers, new downloads or QA-derived subject selection.

Inputs are the retained six public Parquet files and immutable Python pilot at
WORK/coordinator/wikidata-snapshot. Pin its manifest and database SHA256
c3f1197705f131e74f8f7b794d3f08eaef7d3b48e8a312170185e6728bff1313,
10,367,000,576bytes. Source: CC0 Wikidata20260928, publisher permutans/wikidata-*,
firstof86claims shards plusall en/mul names; scholarlyworks excluded. Existing
PARENT/source metadata reports remain authoritative. Never mutate the pilot.

Hypothesis: a native bounded transformation can rebuild all alias keys using the
current Rust normalization, preserve the pilot's graph and provenance, and open
its substantially larger coverage through the native facts query engine without
loading the graph into RAM. This is a resource/ingestion gate, not a QA gain.

Before transformation, independently scan every raw subject/name/property ID and
all selected entity-valued targets. Verify canonical string -> signedinteger ->
string roundtrip, and exact selected row counts. This is necessary because the
Python pilot stored numeric IDs; do not assume leading-zero or range fidelity.
All raw hashes and the immutable database hash must match their prior manifests.
Bound this prerequisite to180seconds and8GiB/no-swap.

Implementation direction: a new versioned native import command can attach the
pilot read-only, rebuild keys using safe Rust SQLite scalar functions, and create
a separate output database. Never reuse Python normalized keys. Insert entities
from names and claim subjects, deduplicate only identical (entity,originalalias)
and (subject,property,object) records, and retain a statement-evidence table with
original statement ID, normal/preferred rank, qualifier-presence and source-row
identity. Raw Parquet remains the source of full qualifiers/references. Keep the
qualification/history and incomplete-shard scope explicit; do not call these
current timeless facts. Default-language andEnglish source identity must remain
traceable. Metadata must distinguish this public-snapshot input from version1
JSON-store input; retain existing version1 query compatibility.

Create secondary indexes after bounded ingestion where useful. Reserve new output
paths, use transactions, and mark partial builds invalid. No officialheldout,
TRAIN/CAL/DEV questions or answers enter graph construction. Preserve source
manifests/license/revision/counters/hash in the generated artifact or paired
manifest so copied databases remain attributable. No existing talkbundle replaced.

Acceptance fixed before implementation:
- Source canonical-roundtrip prerequisite passes on all relevant values and row
  counts, with no discarded or silently changed IDs.
- Completed native database stays below24GiB; import, indexconstruction and
  integrity checks finish within3600seconds under8GiB/no-swap. Additional retained
  run growth stays below32GiB, excluding alreadyexisting sources/buildcache.
- Exact subject/alias/edge accounting against the input; only declared normalization
  eligibility and identical-row deduplication change counts. Retain all5,677,515
  statement witnesses, including707,253qualified rows. Native alias keys match
  the existing Rust normalizer; no Python keys consumed.
- Independent deterministic raw-row samples verify values/source identity; graph
  and alias query plans use indexes. Compare bounded extracted local views with
  the alreadyvalidated JSON/native implementations. Check public-scale query
  success/refusal and memory without claiming expected benchmark answers.
- Existing JSONimport/native-query parity and focused tests remain valid; relevant
  main checks and workspace gates pass before any publication.

If a resource or data prerequisite fails, preserve the partial artifact and change
representation or approach; do not enlarge the gate after seeing results. After
native ingestion passes, freeze a separate strict joint QA experiment on the
approved calibration/development data. The other85claims shards remain unfetched
until a separately justified bounded acquisition plan exists. The original goal
is broader semantic capability, so ingestion is a prerequisite, not its end state.
