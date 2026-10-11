The original strict statement-ID validator required uppercase Q in statement IDs.
The first real-cache CLI load failed with `invalid statement identity/schema/size`.
The retained raw cache contains historical lowercase-q statement IDs. This original
failure is saved in run/reports/original-real-load-failure.txt. Revised only claim-ID
prefix validation to ASCII case-insensitive equality with the canonical subject;
QIDs used as entities/endpoints remain canonical uppercase. UUID case-insensitive
keys now reject contradictory duplicates rather than treating casing as new claims.
Raw IDs are preserved in witnesses. No temporal policy/contract expectation changed.
No other alternative temporal rules or score-changing experiments were attempted.

First clippy run rejected guarded indexing in the production parser and JSON
accesses. These were replaced with checked get/pointer/first access (no blanket
production lint suppression). The immediate follow-up compile exposed two
borrowed-JSON/String comparisons, fixed with explicit as_str comparisons. A later
clippy run flagged unwrap/indexing in integration-test helper functions; allowances
are confined to the test file. Logs clippy-first/second/third preserve each failure.
The original 56 frozen expectations have not been changed.

A strengthened record-count test initially still supplied null records, so schema
validation rejected it before the record-count branch; the more specific expected
error assertion failed. Corrected the authored loader test to supply 100001
structurally deserializable empty records (under the byte cap), reaching the actual
record-count check. Runtime acceptance rules and frozen statuses unchanged.
Original failure remains tests-final.log; superseding complete run uses tests-accepted.log.
