# Native public facts ingestion

The independently selected Wikidata pilot now has a native Rust query index.
`glassmind-facts import-wikidata-pilot` transforms the hash-pinned, read-only
numeric projection into format 2, rebuilding names with the existing Rust
normalizer. The database embeds the original file hashes, revisions, license,
scope and row mappings. Every statement keeps its GUID, rank, qualifier flag
and source-row identity. Deduplicated aliases retain language/kind provenance,
occurrence counts and a first raw-row witness.

This is a storage and query capability, not a QA accuracy improvement. The
source includes qualified and historical statements. Full qualifiers and
references remain in the original Parquet. Only the first lexical claim shard
of 86 was acquired, with all English/default-language names; scholarly works
are excluded by the publisher. No QA annotations selected or populated the graph.
Existing talk bundles, models, scores and default routes are unchanged.

## Frozen gates and results

The [plan](PLAN.md) preceded construction and QA evaluation. The prior canonical
ID audit checked all 109,644,279 relevant raw ID values in 15.595 seconds,
establishing lossless reconstruction from the numeric pilot. It was not repeated.

| Quantity | Verified result |
|---|---:|
| Entities, including aliasless claim subjects | 59,853,388 |
| Original name rows | 84,990,012 |
| Rows with eligible native keys | 84,955,531 |
| Rows excluded by the existing two-byte key boundary | 34,481 |
| Distinct eligible `(entity, original name)` aliases | 74,983,525 |
| Distinct graph triples | 5,659,191 |
| Statement witnesses retained | 5,677,515 |
| Qualified witnesses retained | 707,253 |
| Native database bytes | 12,306,595,840 |
| Import, indexes and integrity, including supervisor overhead | 816.399 seconds |
| Import process peak RSS | 277,036 KiB |
| Sampled peak run footprint, including unlinked sorting files | 20,852,143,958 bytes |

The fixed limits were less than 24 GiB output, less than 32 GiB additional run
space and 3,600 seconds under `MemoryMax=8G`, `MemorySwapMax=0`. All passed.
Scope memory reached its cap through file cache; observations recorded no OOM
events. Process RSS and charged scope memory are different measurements.
SQLite integrity and foreign-key checks completed before the transaction committed.
The completed database SHA-256 is
`f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b`.

The independent audit took 69.283 seconds within its 600-second bound. All 954
fixed raw-row samples matched, including 59 retained full statement witnesses
and 854 raw names. Version 1 normalization independently checked sampled alias
keys and casing. A separate Rust source scan derived exact eligibility and
deduplication counts without reading the new index's counters. Its boundary
calculation matched full normalization for all 1,112,064 Unicode scalars and
compound cases. Independent subject/edge counts and rank/qualifier totals matched.
Alias, forward and inverse plans use indexes.

## Public queries and compatibility

The 853 eligible source-name queries produced 677 unique links and 176 ambiguity
refusals. All 50 confident graph refusals matched the original pilot. These names
alone produced no substantive answers; that limitation remains recorded.
A fresh small launcher measured the query process at 74,136 KiB peak RSS and
0.120 seconds for this one local batch, including startup. This is not a cold-cache
or general latency claim.

The separately frozen [witness probe](WITNESS-PLAN.md) then exercised positive
queries in both directions using the same 59 raw witnesses. Eight anchors lacked
a name; none exceeded query limits. All 110 remaining cases matched source-derived
answers, refusals, lexical set prefixes and complete counts: 37 ordinary answers,
73 ordinary refusals, 59 answer sets and 13 truncated sets, largest count 1,756.
A tiny synthetic relation model prescribes each tested property and is discarded.
It is trained on each probe itself, so these are query API checks, not relation
prediction or QA accuracy results.

An extracted raw view contains 913 entities, 853 aliases and 59 triples. All 853
source-name questions match the existing JSON implementation's full serialized
ordinary and set answers. Separately, the updated reader exactly preserves all
19,481 TRAIN and 2,821 DEV ordinary/set outputs on the unchanged version 1 graph.
Main chat checks remain 28/28 in both plain and intent modes. All 33 facts tests,
workspace formatting, clippy, release build and the strict 4/4 smoke gate passed.

## Review and provenance

Import source is `b44df61`; the final production source adds literal-path handling
in `bba402d`. Review found that a relative filename starting with `file:` could be
interpreted as a SQLite URI. Both importers and the reader now canonicalize literal
paths. The regression proves that the URI's pre-existing target is unchanged.
The full import used absolute paths and is unaffected by this fix; its original
binary is retained separately from the final reader. Other subsequent commits
add helpers and reports only.

Initial `u64` SQLite conversion and clippy failures are retained. The first path
regression caught an incomplete writer-only correction before the reader was
fixed. No data selection, resource gate, normalization rule or acceptance
criterion was weakened. No model worker or external model call was launched,
and no new data was downloaded.
Known completed worker accounting remains 201,926,299 tokens, excluding parent
and interrupted usage, with cached input counted once.

Run artifacts are under
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/wikidata-native`.
[artifacts.json](artifacts.json) pins the binaries, scripts, sampled views,
predictions, reports and database. The six original files and immutable Python
pilot remain in the adjacent `wikidata-snapshot` directory. Scripts are retained
with their executed directory layout; they reserve output names and must not be
blindly rerun over completed outputs. Commands and source manifests are included.

The next experiment must freeze a separate strict entity/relation/answer QA
comparison before running approved calibration or development questions against
this graph. Broader claim acquisition requires its own justified, bounded plan.
