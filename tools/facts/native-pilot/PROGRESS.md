# Native public pilot ingestion: prerequisite passed, transformation pending

The frozen canonical-ID prerequisite passed in15.595seconds under8GiB/no-swap.
All94,498,260 rawsubject/name IDs,9,468,504 property IDs and5,677,515 selected
claimtarget IDs roundtrip through signedinteger storage without change. The raw
file hashes andimmutablepilot database hash match the prior manifests. Exact
Qitem counts are84,990,012names and5,677,515selectedstatements. No QA inputs used.

The first helper failed on Arrow large_string/string scalar type mismatch before
finishing the first file; its original code/log are preserved. The corrected
helper uses a matching large_string separator and handles empty selected batches.
No data gate, row selection, resource limit or expected result was changed.

Next implement the native transformation described in PLAN.md. No new database
has been created. Do not mutate the Pythonpilot or consume its normalized field.
Native code is in crates/facts/src/disk.rs, source3de724e; JSON/native parity passed.

Pilot schema (read-only):
- names(row_id INTEGER PK, entity INTEGER, name TEXT, normalized TEXT,
  language INTEGER, kind INTEGER). language0=en,1=mul; kind0=label,1=alias.
- edges(row_id INTEGER PK, subject INTEGER, property INTEGER, object INTEGER,
  preferred INTEGER, qualified INTEGER, statement TEXT).
Allnumeric IDs can now be reconstructed as the original Q/Pstrings. Namesrow_id
is the global source-row ordinal plus1; file offsets are in the original
wikidata-snapshot/import-report.json. Edgesrow_id is claims source ordinal plus1.

A format2 schema should distinguish public snapshot provenance from the format1
JSON input hash. Keep sourcefile/revision/license/scope/dataset hashes portable;
preserve the JSONformat1 reader/importer. Keep allstatement witnesses separately
from unique graph edges. For deduplicated aliases retain language/kind provenance
(e.g. a source mask, occurrence count and source row reference) so en/mul labels
and aliases remain traceable. Rebuild normalized/casing fields through the same
Rust functions; SQL length tests must use UTF-8 bytes, matching Rust String::len.
Create secondary indexes after import where appropriate. Reuse query semantics,
read-only snapshots andresource bounds, rather than adding another answer engine.

A safe Rust SQLite scalar function plus read-only attachment can rebuild keys
without dumping a huge JSON Store. rusqlite functions feature was deliberately
removed from the completed format1 change because unused; add it only when used
by this transformation. The fixed resource gate is24GiB output,32GiB added run
space and3600seconds under8GiB/no-swap. Onlycachedpilot/rawdata are allowed here.
Strict QA evaluation and wider downloads require separate frozen experiments.
