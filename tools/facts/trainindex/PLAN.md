# Preregistration (before validation access)

2026-10-10. New TRAIN-scoped finite-retrieval series; historical validation-seeded
500-entity index remains untouched and is not a comparable baseline.

Hypothesis: independently fetched Wikidata claims for a frozen 1,000-QID TRAIN
selection improve correct substantive dev answers over its first-100 prefix,
with answered precision decreasing by at most 0.02. Accept only if expanded
right > baseline right, precision >= baseline precision - 0.02, exact standalone/
talk answer-ID equivalence, and functioning relocated opt-in offline bundle.
Report every newly wrong and earlier-correct-lost row; no dev tuning.

Exact selection: read only TRAIN; take rows with zero-based index less than
floor(0.9 * total TRAIN rows), excluding i % 10 == 0 (old relation calibration).
Count only valid subject QIDs. Sort unique QIDs by descending count then ascending
SHA256 of UTF-8 QID. Freeze first 1,000 and nested first 100 BEFORE fetching or
reading validation. This explicitly interprets 'first 90%' as a TRAIN prefix,
then excludes calibration; the extra final 10% is not used for scope selection.
TRAIN relations/objects/questions never create index claims or aliases.

Use unchanged frozen relation model, linker, scorer, and baseline extraction:
nondeprecated entity-valued QID claims, all nondeprecated ranks, no qualifier
filtering. Only selected QIDs may be searchable entities. Existing raw cache
may provide exact selected entities; no old index import. Labels for claim
targets may be reused for display only; fallback IDs are fine.

Sequential public Wikidata API requests: identifying UA, gzip, maxlag=5,
<=25 QIDs/request, >=2 seconds between request starts, <=80 total requests,
<=600 seconds network phase, <=80 MiB new decompressed raw bytes, <=2 attempts,
honor Retry-After within budget and stop repeated limits. Freeze requested
prefixes even if fetch is incomplete; report exact missing IDs and actual counts.
Raw replies and derived index/bundle/predictions stay outside git in own run/tmp.
Record request times, revisions, hashes, sizes, sources and licenses.

Run unchanged score first200 dev then all2821 for both scopes. Question-only
prediction streams are separate from offline annotations; retain full per-row
right/refused/wrong transitions and denominators. Test independent factual
questions and asset binding, schema and relocation. No runtime network/LLM.
Inverse-local uniqueness is not global uniqueness; temporal qualifiers and rank
semantics remain inherited limitations. Rejection retains evidence, no default
replacement. No agents, credentials, API keys, commits or shared-file edits.
