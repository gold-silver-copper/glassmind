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
