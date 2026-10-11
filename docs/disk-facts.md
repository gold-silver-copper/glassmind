# Query a facts index on disk

`glassmind-facts` can import a finite JSON facts store into a native SQLite index
and query it without loading the whole graph. It uses the same Rust alias
normalization, learned relation model and answer rules as the JSON store.

```sh
# Creates a new file; an existing database is never overwritten.
glassmind-facts index-disk --index index.json --database facts.sqlite

glassmind-facts ask-disk --model relation.json --database facts.sqlite \
  'Who wrote TITLE?'

# First ten distinct IDs in lexical order, with the complete local count.
glassmind-facts ask-disk --model relation.json --database facts.sqlite --set \
  'Who wrote TITLE?'

# One question per line, one JSON response per line; --set also works here.
glassmind-facts predict-disk --model relation.json --database facts.sqlite < questions.txt
```

A single answer still requires one unambiguous alias and one local matching ID.
Equal-length alias ties refuse; acronym casing cannot resolve a tie or expose a
shorter alias. Forward and inverse queries deduplicate IDs. Set counts describe
the loaded snapshot, including any limitations of its original claims.

Queries open the database read-only and hold one read snapshot for the lifetime
of the connection. An external writer cannot change the evidence midway through
a batch. The format records the JSON input's SHA-256 and the Rust Unicode
normalizer version; incompatible schemas or normalizers require rebuilding.
The recorded source hash describes the import input, not subsequent external
edits to the database.

This importer accepts JSON up to 32 MiB; it does not yet import raw Wikidata
Parquet or the experimental Python SQLite pilot. Relation model loading is
limited to 64 MiB. A query is limited to 4,096 bytes and 128 normalized tokens.
SQLite uses a 64 MiB page cache, no memory mapping, a 1 MiB row-value limit and
an instruction-progress budget. Exceeding a bound returns an error rather than
a partial answer or an incomplete set count. `predict-disk` preserves blank
lines as queries; an invalid line stops the command with an error.

The existing JSON commands and `glassmind talk --facts-dir` remain available.
This disk backend currently serves the dedicated single-answer and set commands;
conversation, explicit relationship verification and temporal matching still use
their existing bundle paths.

[Compatibility results and provenance](../tools/facts/disk/PARENT.md) cover all
19,481 TRAIN and 2,821 DEV questions on the same 999-entity snapshot. This is
storage equivalence, not an increase in knowledge coverage or QA accuracy.
# Public Wikidata pilot import

`glassmind-facts import-wikidata-pilot --pilot pilot.sqlite --manifest source-manifest.json --database new.sqlite`
converts an independently audited numeric Wikidata projection into native format 2.
The committed example manifest is `tools/facts/native-pilot/source-manifest.json`.
Before using a numeric projection, audit the raw Q/P identifiers for exact
string-to-integer-to-string round trips; the importer cannot recover spelling
already lost upstream. The manifest pins that audit, the pilot hash and size,
and each original public file's revision, hash, row range and retained count.
It records the publisher, snapshot, license and incomplete selection scope.
Raw files resolve under
`https://huggingface.co/datasets/permutans/wikidata-{table}/resolve/{revision}/{path}`.

The importer verifies the pilot hash, attaches it read-only, validates source
row ranges and counts, and creates a separate database in one transaction.
It never reads the Python `normalized` column: Rust rebuilds both normalization
and acronym-case keys. Only normalized aliases shorter than two UTF-8 bytes are
ineligible. Distinct original aliases and graph triples are deduplicated by
their exact values, with no entity selection from QA data.

Format 2 keeps each statement in `statements`, with original GUID, source row,
normal/preferred rank and qualifier presence. The original Parquet remains the
authority for full qualifiers and references. Aliases retain an occurrence count,
first source-row witness, and a bitmask: English label=1, English alias=2,
default-language label=4, default-language alias=8. Row IDs are one plus the
raw row ordinal plus the manifest's file offset. Multiple provenance bits may
apply to one alias. These are recorded relationships, including qualified and
historical statements; they are not assertions that every relationship holds now.

The database embeds the complete manifest and exact import counters. Format 2
reports `public_source` instead of `source_json_sha256`; the latter continues to
describe only version 1 JSON imports. Existing `ask-disk` and `predict-disk`
commands read either format, using the same bounded query engine. Existing talk
bundles and default commands are unchanged.

Use a memory-capped scope (`MemoryMax=8G`, `MemorySwapMax=0`), an external timeout,
and a private on-disk `TMPDIR`/`SQLITE_TMPDIR` while importing. The importer limits
the output to less than 24 GiB and checks a 3,600-second deadline through SQLite's
progress hook. The experiment supervisor also bounds temporary and retained run
growth to 32 GiB. It builds secondary indexes after ingestion and checks integrity
and foreign keys before committing. Existing output files are never overwritten;
preserve interrupted or failed outputs for diagnosis and choose a new path.

The experimental `import-wikidata-claims --names NATIVE_PILOT --claims
NUMERIC_PROJECTION --manifest SOURCE.json --database NEW.sqlite` command reuses
format-2 native alias keys and replaces claims with a separately pinned,
multi-shard projection. Its format-3 provenance embeds the original names source,
both input hashes, projection audit hash, and each claims file's global raw row
range. Each retained statement keeps its original GUID, rank and qualification
flag. Claims and names use separate raw row spaces. Qualified or historical
statements remain recorded evidence, not assertions of timeless/current truth.

Inputs must match their recorded sizes and SHA256 hashes. The output must be a
new literal path; a failed transaction never receives a readable format header.
The command builds all indexes and checks integrity and references before commit.
Its separate resource limits are 192 GiB output and six hours; run it under the
required 8 GiB/no-swap scope and an external disk/time supervisor. These broader
limits do not change the original pilot's limits. Existing format-1 and format-2
query behavior remains supported. Full-release ingestion and source validation
are still in progress; this command does not promote a QA bundle.
