# Independent Wikidata storage pilot: passed

Parent experiment, 2026-10-11. This establishes acquisition and storage
feasibility, **not answer accuracy**. No QA questions, answer IDs, calibration,
DEV or held-out data entered the experiment. No production runtime, model,
bundle, facts score or default selection changed.

The fixed plan chose the first claims shard by file path and all English/default
language names from the independently published 20260928 Wikidata release.
The publisher excludes scholarly works. CC0-1.0 cards, exact repository revisions,
file trees and expected LFS SHA256 values are preserved in `source-metadata/`
and `pilot-manifest.json`. The broader graph has 86 claims shards totaling
44,633,986,614 bytes; only one was acquired. No remaining download is running.

All six files, totaling **1,475,606,067 bytes**, matched publisher hashes.
Acquisition took 56.557 seconds, with one successful attempt per file.
`fetch.jsonl` records every attempt. An offline review subsequently corrected
HTTP-date Retry-After handling to stop on long/unknown delays; absent, short
numeric, long numeric, long HTTP-date and invalid-header cases pass. No rate
response occurred in the actual acquisition; the originally executed fetch
script is retained in `executed-source/`.

| Measurement | Result |
|---|---:|
| Source claims inspected | 9,468,504 |
| Retained entity-to-entity statements | 5,677,515 |
| Subjects with retained statements | 1,094,802 |
| Retained names, including source duplicates | 84,990,012 |
| All source rows checked for IDs/language | 94,498,260 |
| Invalid IDs, property IDs or name languages | 0 |
| Raw row samples independently checked | 954 |
| SQLite database bytes | 10,367,000,576 |
| Import, indexes, integrity check and summaries | 602.322 seconds |
| Peak process resident memory | 602,132 KiB |

The import and audits ran in 8 GiB memory-capped systemd scopes with no swap,
own process groups, explicit timeouts and disk-backed temporary storage.
The import scope reached its 8 GiB cgroup peak including file cache; the process
RSS above is a different measurement. The frozen database limit was 12 GiB,
and the import limit was 1,800 seconds. Both passed.

Claims excluded by the fixed policy: 324,171 non-item subjects, 3,419,027
non-item values, 11,939 deprecated statements, 34,893 unknown-value statements
and 959 no-value statements. Retained rows preserve the original statement ID,
preferred/normal rank and a qualifier-presence flag. Of retained statements,
707,253 have qualifiers. The raw Parquet retains their full qualifiers and
references; this pilot database is not a temporal truth representation.
Claim IDs span the lexically first shard; the last raw subject is `Q10280514`.
This shard is not a random or complete knowledge graph.

SQLite `integrity_check` returned `ok`. The independent audit reread raw Parquet,
including complete qualifier structures rather than the importer's projected
keys, and checked retained and excluded row samples, exact values, source row
identities, forward/inverse membership and name lookup. All 954 samples pass.
Exact whole-source ID/property/language checks also pass. Query plans use
`edges_forward`, `edges_inverse` and `names_lookup`; lookups are indexed.
This does not independently rederive every retained claim from scratch.

The first import stopped before insertion because its statement GUID check
assumed an uppercase entity prefix. A raw record and
[Wikibase's GUID documentation](https://doc.wikimedia.org/Wikibase/REL1_38/php/classWikibase_1_1DataModel_1_1Statement_1_1StatementGuid.html)
confirmed legacy lowercase prefixes. The corrected check compares only that
prefix case-insensitively and preserves the complete original GUID. The first
script, log, empty database identity and corrected executed source are retained.

## Required next step before runtime use

The pilot's Python normalization is **not compatible with the current Rust
linker for all Unicode**. Testing every Unicode scalar between `A` and `B`
found 6,167 differences (Rust Unicode 17 versus Python Unicode 16, plus different
alphabetic classification). The exact probe and result are retained. This is a
structural compatibility check, not a count of affected real names.

Implement a bounded native disk-backed facts adapter with the same Rust
normalizer for ingestion and queries. First reproduce existing finite-store
linking, ambiguity, acronym casing, forward/inverse sets and refusals on the
same data. Rebuild name keys natively; do not simply attach this pilot database
to the runtime. Keep graph ranks, qualification and incomplete coverage explicit.
Only after adapter parity should a separately frozen broader-data experiment
choose resource bounds and strict joint QA acceptance gates. Freeze that separate
plan before downloading remaining claims or evaluating broader QA coverage.

Artifacts: `WORK/coordinator/wikidata-snapshot`, where
`WORK=/home/gsc/.cache/glassmind/continuous-20261010`. The read-only pilot database
has SHA256 `c3f1197705f131e74f8f7b794d3f08eaef7d3b48e8a312170185e6728bff1313`.
Raw data and the local PyArrow 25.0.1 venv stay outside git. The repository
contains scripts, plans, source metadata, commands, hashes and reports.
`commands.txt` records scoped invocations; `import.py` refuses to overwrite a
database. Workspace fmt/clippy/release and the strict four-turn smoke pass.
All acquisition/import/audit/check jobs are complete; no model worker ran.
