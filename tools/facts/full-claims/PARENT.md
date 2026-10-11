# Full independent Wikidata claims import

The native disk importer now accepts a complete, audited multi-file claims
projection and reuses the already native alias index. This removes the previous
single-shard storage restriction. It does not change alias normalization, relation
prediction, query budgets, conversation bundles or the default QA graph.

`import-wikidata-claims --names NATIVE_PILOT --claims NUMERIC_PROJECTION
--manifest SOURCE_MANIFEST --database NEW_DATABASE` creates format3. Both inputs
are hash/size pinned and attached read-only. Source row ranges, per-file and total
counts, integer identifiers, statement subject/GUID consistency, rank and qualifier
flags are checked before publication. The transaction preserves native alias keys,
deduplicates graph triples and retains all statement witnesses. Integrity and
foreign-key checks precede the final format header and commit. Existing outputs
are never overwritten. The reader remains compatible with formats1 and2.

The source is the independent permutans CC0-1.0 Wikidata20260928 claims release,
revision39e6837bef8b6f982fbf23acd19134e882400db7, all86files excluding scholarly works.
The frozen inventory preceded acquisition and any newgraph QA. No entities were
selected using benchmark answers. All789435443rawrows were projected under fixed
bounds; the canonical audit verified1092134757retained Q/P IDs losslessly. Original
names provenance is inherited from the separately audited native pilot. Original
Parquet remains authoritative for complete qualifiers/references. Qualified and
historical statements are not assertions that relationships hold now.

The resulting native database has74445176entities,74983525aliases,362302267distinct
edges and364044919statement witnesses, including46448050qualified statements.
It occupies62772387840bytes and is read-only in the retained run directory:
SHA256 fc04ca4ac6f30468f619395a310181e23ebb45a44f776b81bb74178b357f0172.
The importer took3052.182s, peaking at371540KiBprocessRSS and142749644072sampled
added-run bytes, below the original21600s/192GiBoutput/384GiBaddedrun/8GiBmemory
caps. All experiments used no swap and on-disk temporary storage.

Independent raw references were pinned before native ingestion. All8009prescribed
samples across86files agree with both the numeric projection and native witnesses;
3647are eligible and excluded samples remain absent. Independent distinct-triple,
entity-union, alias-occurrence and rank/qualifier totals agree. All854raw-derived
alias samples match. Forward/inverse/alias query plans use indexes. Combined raw
reference and final audit took671.175s within the original3600s gate.

Source-derived probes use the first eligible raw sample in each shard, both
relations, and the first source name without QA selection. Fifteen unnamed anchors
leave157cases, including77alias ambiguities. Independent full-projection scans
supply cardinalities and first10 IDs. All49ordinary answers and79successful sets
match, with108ordinary refusals and24truncated sets. The largest successful set
has1530765IDs. One query with8073242expected matches hits the unchanged SQLite
instruction budget and is separately recorded as an error. It receives no success
credit. The complete supervised probe ran54.040s with123832KiBprocessRSS.

Regression evidence:36facts tests, old19481TRAIN+2821DEV ordinary/set outputs
byte-identical on the unchanged format1graph, all853format2source-name queries
byte-identical,110oldwitness probes unchanged, and28mainmixed cases in each of
plain/intent modes. Focused tests exercise multiple files, omitted source-row gaps,
duplicate triples, GUIDs, native Unicode aliases, forward/inverse answers, source
hashes, malformed records/provenance and literal URI-looking paths. Workspace
fmt/clippy/release and strict4/4smoke passed on the facts branch; central integration
checks are recorded separately.

These are storage and query correctness results, not English QA accuracy. The
single-shard public QA gate remains rejected. Any fullgraph QA experiment requires
its own frozen CAL/DEV criteria before inference; no default promotion is implied.
Artifacts and exact commands are in this directory and the retained external run
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/full-claims`.
